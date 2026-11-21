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
        self.journal = self.root / 'deployments/testnet.journal.json'
        digest = hashlib.sha256(b'code').hexdigest()
        self.manifest.write_text(json.dumps({
            'network': 'testnet', 'fee_to_setter': 'GSETTER',
            'contracts': {'factory': {'id': 'CFACTORY', 'wasm_sha256': digest, 'deploy_tx': '1' * 64, 'initialize_tx': '1' * 64},
                          'router': {'id': 'CROUTER', 'wasm_sha256': digest, 'deploy_tx': '2' * 64, 'initialize_tx': '2' * 64},
                          'pair': {'wasm_hash': '0' * 64, 'upload_tx': '3' * 64}}}))

    def run_script(self, script, *args):
        return subprocess.run(['bash', str(self.root / 'scripts' / script), *args],
                              env=self.env, capture_output=True, text=True)

    def assert_no_submission(self, allow_source=False):
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()] if self.calls.exists() else []
        for call in calls:
            if call[:2] == ['contract', 'invoke']:
                self.assertIn('--send=no', call)
            self.assertNotIn('--send=yes', call)
            if not allow_source:
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
        # Anonymous verification must also work on macOS Bash 3 with nounset.
        self.env['PAIR_COUNT_FAILURE'] = '1'
        result = self.run_script('verify-deployment.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('factory.all_pairs_length unavailable', result.stdout)
        self.assert_no_submission()

    def test_verifier_optional_source_stays_read_only(self):
        result = self.run_script('verify-deployment.sh', '--source', 'mock-identity')
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
        reads = [call for call in calls if 'invoke' in call]
        self.assertTrue(reads)
        for call in reads:
            self.assertIn('--send=no', call)
            self.assertEqual(call[call.index('--source') + 1], 'mock-identity')
        self.assert_no_submission(allow_source=True)

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

    # --- Durable deployment journal (issue #13) ---------------------------

    def prepare_deploy(self):
        self.manifest.unlink()
        (self.root / 'scripts/build.sh').write_text('#!/usr/bin/env bash\nexit 0\n')
        wasm = self.root / 'target/wasm32v1-none/release'
        wasm.mkdir(parents=True)
        for contract in ['factory', 'pair', 'router']:
            (wasm / f'veltrixae_{contract}.wasm').write_bytes(b'code')

    def deploy(self):
        return self.run_script('deploy-testnet.sh', '--source', 'mock-identity',
                               '--i-authorize-testnet-deploy')

    def journal_record(self):
        self.assertTrue(self.journal.exists(), 'journal was not written')
        return json.loads(self.journal.read_text())

    def assert_journal_public_only(self):
        text = self.journal.read_text()
        for secret in ['mock-identity', 'SECRET', 'secret_key', 'seed', 'keystore']:
            self.assertNotIn(secret, text)

    def test_journal_preserved_after_upload_interrupt(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'upload:pair'
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.manifest.exists())
        record = self.journal_record()
        self.assertEqual(record['network'], 'testnet')
        self.assertEqual(record['status'], 'incomplete')
        pair = record['steps']['upload:pair']
        self.assertEqual(pair['wasm_sha256'], hashlib.sha256(b'code').hexdigest())
        self.assertEqual(pair['receipt']['tx_hash'], '1' * 64)
        self.assert_journal_public_only()

    def test_journal_preserved_after_factory_deploy_interrupt(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'deploy:factory'
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.manifest.exists())
        record = self.journal_record()
        factory = record['steps']['deploy:factory']
        self.assertEqual(factory['contract_id'], 'CFACTORY')
        self.assertEqual(factory['receipt']['tx_hash'], '2' * 64)
        self.assertEqual(factory['receipt']['operation'], 'deploy')
        self.assert_journal_public_only()

    def test_journal_preserved_after_router_deploy_interrupt(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'deploy:router'
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.manifest.exists())
        record = self.journal_record()
        self.assertIn('deploy:factory', record['steps'])
        router = record['steps']['deploy:router']
        self.assertEqual(router['contract_id'], 'CROUTER')
        self.assertEqual(router['receipt']['tx_hash'], '2' * 64)
        self.assert_journal_public_only()

    def test_journal_writes_are_atomic(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'upload:pair'
        self.deploy()
        leftovers = [p.name for p in self.journal.parent.iterdir()
                     if p.name.startswith('testnet.journal.json.')]
        self.assertEqual(leftovers, [])

    def test_malformed_journal_fails_clearly(self):
        self.prepare_deploy()
        self.journal.write_text('{not json')
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('journal', result.stderr.lower())
        self.assertFalse(self.manifest.exists())

    def test_stale_journal_network_fails_clearly(self):
        self.prepare_deploy()
        self.journal.write_text(json.dumps({'network': 'mainnet', 'steps': {}}))
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('journal', result.stderr.lower())
        self.assertFalse(self.manifest.exists())

    def test_journal_never_overwrites_existing_manifest(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'upload:pair'
        self.deploy()
        self.manifest.write_text('{"sentinel": true}')
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('refusing redeployment', result.stderr)
        self.assertEqual(self.manifest.read_text(), '{"sentinel": true}')

    def test_journal_rejects_unrelated_receipt_operation(self):
        self.prepare_deploy()
        self.env['RECEIPT_OPERATION'] = 'invoke'
        self.env['INTERRUPT_AFTER'] = 'upload:pair'
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('receipt', result.stderr.lower())
        self.assertFalse(self.manifest.exists())

    def test_journal_rejects_mismatched_contract_identity(self):
        self.prepare_deploy()
        self.env['RECEIPT_CONTRACT'] = 'CWRONG'
        self.env['INTERRUPT_AFTER'] = 'deploy:factory'
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('receipt', result.stderr.lower())
        self.assertFalse(self.manifest.exists())

    def test_journal_rejects_mismatched_code_identity(self):
        self.prepare_deploy()
        self.env['RECEIPT_WASM'] = 'f' * 64
        self.env['INTERRUPT_AFTER'] = 'upload:pair'
        result = self.deploy()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('receipt', result.stderr.lower())
        self.assertFalse(self.manifest.exists())

    def test_recovery_report_lists_completed_and_missing(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'deploy:factory'
        self.deploy()
        result = self.run_script('recover-testnet.sh')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('deploy:factory', result.stdout)
        self.assertIn('deploy:router', result.stdout)
        self.assertIn('missing', result.stdout.lower())
        self.assert_no_submission()

    def test_recovery_report_missing_journal_fails(self):
        result = self.run_script('recover-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.calls.exists())

    def test_recovery_report_malformed_journal_fails(self):
        self.journal.write_text('{not json')
        result = self.run_script('recover-testnet.sh')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('journal', result.stderr.lower())
        self.assertFalse(self.calls.exists())

    def test_verifier_reconciles_journal_receipts(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'deploy:factory'
        self.deploy()
        result = self.run_script('verify-deployment.sh', '--journal')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('deploy:factory', result.stdout)
        self.assert_no_submission()

    def test_verifier_rejects_journal_receipt_mismatch(self):
        self.prepare_deploy()
        self.env['INTERRUPT_AFTER'] = 'deploy:factory'
        self.deploy()
        record = self.journal_record()
        record['steps']['deploy:factory']['contract_id'] = 'CWRONG'
        self.journal.write_text(json.dumps(record))
        result = self.run_script('verify-deployment.sh', '--journal')
        self.assertNotEqual(result.returncode, 0)
        self.assert_no_submission()


if __name__ == '__main__':
    unittest.main(verbosity=2)
