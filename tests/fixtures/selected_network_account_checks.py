"""Exercise the actual CLI against isolated RPC servers and a real terminal."""
import errno
import http.server
import json
import os
import pathlib
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import threading
import time
import unittest
import fcntl
import termios

BINARY = sys.argv.pop(1)
FIXTURE = json.load(sys.stdin)
HASH = "11111111111111111111111111111111"


class Rpc:
    def __init__(self, account=True, key=True, failures=0):
        self.account = account
        self.key = key
        self.failures = failures
        self.requests = []
        owner = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_POST(self):
                req = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                owner.requests.append(req)
                params = req["params"]
                kind = params.get("request_type")
                account_id = params.get("account_id")
                result = {"block_hash": HASH, "block_height": 10}
                error = None
                if owner.failures:
                    owner.failures -= 1
                    error = {"name": "INTERNAL_ERROR", "cause": {"name": "INTERNAL_ERROR", "info": {}}, "code": -32603, "message": "fixture RPC failure"}
                elif kind == "view_account":
                    present = owner.account(account_id) if callable(owner.account) else owner.account
                    if present:
                        result.update(amount="1000000000000000000000000", locked="0", code_hash=HASH, storage_usage=0)
                    else:
                        error = {"name": "HANDLER_ERROR", "cause": {"name": "UNKNOWN_ACCOUNT", "info": {"requested_account_id": account_id, "block_height": 10, "block_hash": HASH}}, "code": -32000, "message": "Server error"}
                elif kind == "view_access_key":
                    if owner.key:
                        result.update(nonce=1, permission="FullAccess")
                    else:
                        error = {"name": "HANDLER_ERROR", "cause": {"name": "UNKNOWN_ACCESS_KEY", "info": {"account_id": account_id, "public_key": params["public_key"], "block_height": 10, "block_hash": HASH}}, "code": -32000, "message": "Server error"}
                else:
                    error = {"code": -32601, "message": "Unexpected fixture request"}
                response = {"jsonrpc": "2.0", "id": req["id"]}
                response["error" if error else "result"] = error or result
                data = json.dumps(response).encode()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.url = "http://127.0.0.1:%d/" % self.server.server_port

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()


class Terminal:
    def __init__(self, args, env, cwd):
        self.master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 160, 0, 0))
        self.process = subprocess.Popen([BINARY] + args, stdin=slave, stdout=slave, stderr=slave, env=env, cwd=cwd, start_new_session=True)
        os.close(slave)
        self.output = ""
        self.cursor = 0

    def read(self):
        if select.select([self.master], [], [], 0.1)[0]:
            try:
                data = os.read(self.master, 65536)
            except OSError as error:
                if error.errno == errno.EIO:
                    return
                raise
            # Respond to terminal cursor-position queries used by inquire.
            if b"\x1b[6n" in data:
                os.write(self.master, b"\x1b[1;1R")
            self.output += re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode(errors="replace"))

    def expect(self, text):
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            self.read()
            index = self.output.find(text, self.cursor)
            if index >= 0:
                self.cursor = index + len(text)
                return
            if self.process.poll() is not None:
                break
        raise AssertionError("Missing prompt %r:\n%s" % (text, self.output))

    def send(self, value):
        os.write(self.master, value)

    def finish(self):
        deadline = time.monotonic() + 30
        while self.process.poll() is None and time.monotonic() < deadline:
            self.read()
        if self.process.poll() is None:
            raise AssertionError("CLI did not exit:\n" + self.output)
        self.read()
        return self.output

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait()
        os.close(self.master)


class SelectedNetwork(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.selected = Rpc()
        self.other = Rpc()
        self.addCleanup(self.selected.close)
        self.addCleanup(self.other.close)
        credentials = self.root / "credentials"
        credentials.mkdir()
        (credentials / "ft_contracts.json").write_text("[]")
        config_dir = self.root / ("Library/Application Support/near-cli" if sys.platform == "darwin" else "near-cli")
        config_dir.mkdir(parents=True)
        # The unrelated alias comes first and shares network_name with the chosen connection.
        config = 'version = "5"\ncredentials_home_dir = %s\n' % json.dumps(str(credentials))
        for name, rpc in [("other", self.other), ("chosen", self.selected)]:
            config += '\n[network_connection.%s]\nnetwork_name = "same-network"\nrpc_url = "%s"\nwallet_url = "%s"\nexplorer_transaction_url = "%s"\n' % (name, rpc.url, rpc.url, rpc.url)
        (config_dir / "config.toml").write_text(config)
        self.env = dict(os.environ, HOME=str(self.root), XDG_CONFIG_HOME=str(self.root), APPDATA=str(self.root), TERM="xterm", NO_COLOR="1")
        # Local fixtures must not be routed through a machine's HTTP proxy.
        self.env["NO_PROXY"] = self.env["no_proxy"] = "127.0.0.1,localhost"

    def tearDown(self):
        self.assertEqual(self.other.requests, [], "Unselected RPC was contacted")

    def imported(self, account="alice.testnet", check=True):
        args = ["account", "import-account", "using-private-key", FIXTURE["secret"], account, "network-config", "chosen"]
        if check:
            args += ["--check-account-id"]
        return args + ["save-to-legacy-keychain"]

    def run_cli(self, args):
        return subprocess.run([BINARY] + args, env=self.env, cwd=self.root, text=True, capture_output=True, timeout=20)

    def terminal(self, args):
        term = Terminal(args, self.env, self.root)
        self.addCleanup(term.close)
        return term

    def kinds(self):
        return [request["params"].get("request_type") for request in self.selected.requests]

    def test_import_present(self):
        result = self.run_cli(self.imported())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.kinds(), ["view_account", "view_access_key"])

    def test_import_missing_selected_but_present_elsewhere(self):
        self.selected.account = False
        result = self.run_cli(self.imported())
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("same-network", result.stderr)
        self.assertEqual(self.kinds(), ["view_account"])

    def test_import_missing_key(self):
        self.selected.key = False
        result = self.run_cli(self.imported())
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Couldn't find access key", result.stderr)
        self.assertEqual(self.kinds(), ["view_account", "view_access_key"])

    def test_import_uninitialized_implicit(self):
        self.selected.account = False
        result = self.run_cli(self.imported(FIXTURE["implicit"]))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.kinds(), ["view_account"])

    def test_import_mismatched_implicit(self):
        self.selected.account = False
        result = self.run_cli(self.imported("ab" * 32))
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.kinds(), ["view_account"])

    def test_import_initialized_implicit_checks_key(self):
        self.selected.key = False
        result = self.run_cli(self.imported(FIXTURE["implicit"]))
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.kinds(), ["view_account", "view_access_key"])

    def test_import_without_check(self):
        result = self.run_cli(self.imported(check=False))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.kinds(), [])

    def test_import_interactive_yes(self):
        term = self.terminal(self.imported(check=False)[:-1])
        term.expect("Would you like to check")
        term.send(b"\r")
        term.expect("How to save an imported account")
        term.send(b"\x1b[B\r")
        term.finish()
        self.assertEqual(self.kinds(), ["view_account", "view_access_key"])
        self.assertEqual(term.process.returncode, 0)

    def test_import_rpc_error_retry(self):
        self.selected.failures = 6
        term = self.terminal(self.imported())
        term.expect("Do you want to try again?")
        term.send(b"\r")
        term.finish()
        self.assertEqual(term.process.returncode, 0)
        self.assertEqual(self.kinds(), ["view_account"] * 7 + ["view_access_key"])

    def meta(self, explicit=False, offline=False):
        args = (["--offline"] if offline else []) + ["transaction", "send-meta-transaction", "base64-signed-meta-transaction", FIXTURE["delegate"], "sign-as"]
        if explicit:
            args += ["relay.testnet", "network-config", "chosen"]
        return args

    def enter_relayer(self, term):
        term.expect("What is the relayer account ID?")
        term.send(b"relay.testnet\r")
        term.expect("What is the name of the network?")
        self.assertEqual(self.kinds(), [], "Account was checked before choosing a network")
        # Config insertion order puts `other` first.
        term.send(b"\x1b[B\r")

    def cancel_signing(self, term):
        term.expect("Select a tool for signing the transaction")
        term.send(b"\x1b")
        return term.finish()

    def test_meta_selected(self):
        term = self.terminal(self.meta())
        self.enter_relayer(term)
        self.cancel_signing(term)
        self.assertEqual(self.kinds(), ["view_account"])

    def test_meta_explicit_bypass(self):
        term = self.terminal(self.meta(explicit=True))
        self.cancel_signing(term)
        self.assertEqual(self.kinds(), [])

    def test_meta_offline_bypass(self):
        term = self.terminal(self.meta(offline=True))
        self.enter_relayer(term)
        self.cancel_signing(term)
        self.assertEqual(self.kinds(), [])

    def test_meta_reentry_and_cancel_retains_latest(self):
        self.selected.account = lambda account: account == "second.testnet"
        term = self.terminal(self.meta())
        self.enter_relayer(term)
        term.expect("Do you want to enter another relayer account id?")
        term.send(b"\r")
        term.expect("What is the relayer account ID?")
        term.send(b"second.testnet\r")
        output = self.cancel_signing(term)
        self.assertIn("sign-as second.testnet network-config chosen", output)
        self.assertEqual([r["params"]["account_id"] for r in self.selected.requests], ["relay.testnet", "second.testnet"])

    def test_meta_missing_use_anyway(self):
        self.selected.account = False
        term = self.terminal(self.meta())
        self.enter_relayer(term)
        term.expect("Do you want to enter another relayer account id?")
        term.send(b"\x1b[B\r")
        self.cancel_signing(term)
        self.assertEqual(self.kinds(), ["view_account"])

    def test_meta_reentry_cancel_retains_latest_missing(self):
        self.selected.account = False
        term = self.terminal(self.meta())
        self.enter_relayer(term)
        term.expect("Do you want to enter another relayer account id?")
        term.send(b"\r")
        term.expect("What is the relayer account ID?")
        term.send(b"second.testnet\r")
        term.expect("Do you want to enter another relayer account id?")
        term.send(b"\r")
        term.expect("What is the relayer account ID?")
        term.send(b"\x1b")
        output = term.finish()
        self.assertIn("sign-as second.testnet network-config chosen", output)
        self.assertEqual([r["params"]["account_id"] for r in self.selected.requests], ["relay.testnet", "second.testnet"])

    def test_meta_rpc_retry(self):
        self.selected.failures = 6
        term = self.terminal(self.meta())
        self.enter_relayer(term)
        term.expect("Do you want to try again?")
        term.send(b"\r")
        self.cancel_signing(term)
        self.assertEqual(self.kinds(), ["view_account"] * 7)

    def test_import_rpc_cancel_is_not_absence(self):
        self.selected.failures = 6
        term = self.terminal(self.imported(FIXTURE["implicit"]))
        term.expect("Do you want to try again?")
        term.send(b"\x1b")
        output = term.finish()
        self.assertNotEqual(term.process.returncode, 0)
        self.assertNotIn("Couldn't find account", output)
        self.assertEqual(self.kinds(), ["view_account"] * 6)

    def test_meta_rpc_skip(self):
        self.selected.failures = 6
        term = self.terminal(self.meta())
        self.enter_relayer(term)
        term.expect("Do you want to try again?")
        term.send(b"\x1b[B\r")
        self.cancel_signing(term)
        self.assertEqual(self.kinds(), ["view_account"] * 6)


if __name__ == "__main__":
    unittest.main(verbosity=2)
