"""Offline guard tests; fake CLI has no network or signing capability."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class Guards(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        shutil.copytree(ROOT / 'scripts', self.root / 'scripts')
        (self.root / 'deployments').mkdir()
        self.calls = self.root / 'calls.jsonl'
        cli = self.root / 'stellar'
        cli.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
a=sys.argv[1:]
with open(os.environ['CALLS'], 'a') as f: f.write(json.dumps(a)+'\\n')
if a[:2] == ['contract','fetch']:
 pathlib.Path(a[a.index('--out-file')+1]).write_bytes(b'code')
elif 'fee_to_setter' in a:
 if os.environ.get('STATE_READ_FAILURE'): raise SystemExit(1)
 print(json.dumps(os.environ.get('ACTUAL_SETTER', 'GSETTER')))
elif 'get_factory' in a: print(json.dumps('CFACTORY'))
elif 'all_pairs_length' in a:
 if os.environ.get('PAIR_COUNT_FAILURE'): raise SystemExit(1)
 print(os.environ.get('PAIR_COUNT_OUTPUT', '0'))
elif a[:2] == ['keys','address']: print('GSETTER')
elif a[:2] == ['contract','upload']:
 print('0' * 64)
 print('1' * 64, file=sys.stderr)
elif a[:2] == ['contract','deploy']:
 assert a.index('--source') < a.index('--')
 assert a.index('--rpc-url') < a.index('--')
 if 'factory' in a[a.index('--wasm')+1]:
  assert '--setter' in a and '--pair_wasm_hash' in a
 else: assert '--factory' in a
 print('CFACTORY' if 'factory' in a[a.index('--wasm')+1] else 'CROUTER')
 print('2' * 64, file=sys.stderr)
elif 'initialize' in a:
 # Reject the original bug: CLI options after the method separator.
 assert a.index('--source') < a.index('--')
 assert a.index('--rpc-url') < a.index('--')
 print('3' * 64, file=sys.stderr)
else: raise SystemExit(1)
''')
        cli.chmod(0o755)
        # Intercept all HTTP requests: these tests never use a public network.
        curl = self.root / 'curl'
        curl.write_text('''#!/usr/bin/env python3
import json, sys
url = next(arg for arg in sys.argv[1:] if arg.startswith('https://'))
if url == 'https://soroban-testnet.stellar.org':
 print(json.dumps({'result': {'passphrase': 'Test SDF Network ; September 2015'}}))
else:
 print(json.dumps({'hash': url.rsplit('/', 1)[-1], 'successful': True}))
''')
        curl.chmod(0o755)
        self.env = dict(os.environ, STELLAR=str(cli), CALLS=str(self.calls))
        self.env['PATH'] = str(self.root) + os.pathsep + os.environ['PATH']
        self.manifest = self.root / 'deployments/testnet.json'
        digest = hashlib.sha256(b'code').hexdigest()
        self.manifest.write_text(json.dumps({
            'network': 'testnet', 'fee_to_setter': 'GSETTER',
            'contracts': {'factory': {'id': 'CFACTORY', 'wasm_sha256': digest, 'deploy_tx': '1' * 64, 'initialize_tx': '1' * 64},
                          'router': {'id': 'CROUTER', 'wasm_sha256': digest, 'deploy_tx': '2' * 64, 'initialize_tx': '2' * 64},
                          'pair': {'wasm_hash': '0' * 64, 'upload_tx': '3' * 64}}}))

    def run_script(self, script, *args):
        return subprocess.run(['bash', str(self.root / 'scripts' / script), *args],
                              env=self.env, capture_output=True, text=True)

    def assert_no_submission(self):
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()] if self.calls.exists() else []
        for call in calls:
            if call[:2] == ['contract', 'invoke']:
                self.assertIn('--send=no', call)
            self.assertNotIn('--send=yes', call)
            self.assertNotIn('--source', call)
            self.assertIn('https://soroban-testnet.stellar.org', call)
            self.assertIn('Test SDF Network ; September 2015', call)

    def test_deployment_cli_options_precede_method_separator(self):
        self.manifest.unlink()
        (self.root / 'scripts/build.sh').write_text('#!/usr/bin/env bash\nexit 0\n')
        wasm = self.root / 'target/wasm32v1-none/release'
        wasm.mkdir(parents=True)
        for contract in ['factory', 'pair', 'router']:
            (wasm / f'veltrixae_{contract}.wasm').write_bytes(b'code')
        result = self.run_script('deploy-testnet.sh', '--source', 'mock-identity',
                                 '--i-authorize-testnet-deploy')
        self.assertEqual(result.returncode, 0, result.stderr)
        record = json.loads(self.manifest.read_text())
        self.assertEqual(record['deployment_model'], 'atomic-constructor-v1')
        for contract in ['factory', 'router']:
            self.assertEqual(record['contracts'][contract]['initialize_tx'],
                             record['contracts'][contract]['deploy_tx'])
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
        for call in calls:
            self.assertNotIn('initialize', call)
            if 'fee_to_setter' in call or 'get_factory' in call:
                self.assertIn('--send=no', call)

    def test_deployment_refuses_existing_manifest(self):
        result = self.run_script('deploy-testnet.sh', '--source', 'mock-identity',
                                 '--i-authorize-testnet-deploy')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('refusing redeployment', result.stderr)

    def test_verifier_valid_record_is_read_only(self):
        result = self.run_script('verify-deployment.sh')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('factory.all_pairs_length = 0', result.stdout)
        self.assert_no_submission()

    def test_verifier_unavailable_pair_count_fails(self):
        self.env['PAIR_COUNT_FAILURE'] = '1'
        result = self.run_script('verify-deployment.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('factory.all_pairs_length unavailable', result.stdout)
        self.assert_no_submission()

    def test_verifier_malformed_pair_count_fails(self):
        self.env['PAIR_COUNT_OUTPUT'] = 'not-a-count'
        result = self.run_script('verify-deployment.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('factory.all_pairs_length returned a malformed count', result.stdout)
        self.assert_no_submission()

    def test_mainnet_rejected_before_cli(self):
        result = self.run_script('deploy-testnet.sh', '--source', 'name', '--network', 'mainnet')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.calls.exists())

    def test_existing_initialized_contracts_are_skipped(self):
        result = self.run_script('remediate-testnet.sh')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('factory=0 router=0', result.stderr)
        self.assert_no_submission()

    def test_missing_manifest_fails_without_deployment(self):
        self.manifest.unlink()
        result = self.run_script('remediate-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.calls.exists())

    def test_unexpected_initialization_fails_closed(self):
        self.env['ACTUAL_SETTER'] = 'GATTACKER'
        result = self.run_script('remediate-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assert_no_submission()

    def test_constructor_manifest_never_recovers_via_initialize(self):
        record = json.loads(self.manifest.read_text())
        record['deployment_model'] = 'atomic-constructor-v1'
        self.manifest.write_text(json.dumps(record))
        self.env['STATE_READ_FAILURE'] = '1'
        result = self.run_script('remediate-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('do not initialize', result.stderr)
        self.assert_no_submission()
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
        self.assertFalse(any('initialize' in call for call in calls))

    def test_wrong_code_fails_closed(self):
        record = json.loads(self.manifest.read_text())
        record['contracts']['factory']['wasm_sha256'] = '0' * 64
        self.manifest.write_text(json.dumps(record))
        result = self.run_script('remediate-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assert_no_submission()

    def test_wrong_manifest_network_fails_before_cli(self):
        record = json.loads(self.manifest.read_text())
        record['network'] = 'mainnet'
        self.manifest.write_text(json.dumps(record))
        result = self.run_script('remediate-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.calls.exists())


if __name__ == '__main__':
    unittest.main(verbosity=2)
