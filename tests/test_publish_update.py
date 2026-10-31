import json, pathlib, sys, tempfile, unittest, datetime
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "scripts"))
import publish_update as pu


class ManifestTests(unittest.TestCase):
    def test_manifest_shape(self):
        now = datetime.datetime(2026, 10, 31, 12, 0, 0, tzinfo=datetime.timezone.utc)
        m = pu.build_manifest("2.1.1", "notes", "SIG", "https://x/y.exe", now=now)
        self.assertEqual(m["version"], "2.1.1")
        self.assertEqual(m["pub_date"], "2026-10-31T12:00:00Z")
        self.assertEqual(m["platforms"]["windows-x86_64"], {"signature": "SIG", "url": "https://x/y.exe"})

    def test_manifest_has_no_ids_or_telemetry_fields(self):
        m = pu.build_manifest("2.1.1", "", "S", "u")
        self.assertEqual(set(m), {"version", "notes", "pub_date", "platforms"})

    def test_bad_version_rejected(self):
        for v in ("2.1", "v2.1.1", "latest", ""):
            with self.assertRaises(ValueError):
                pu.build_manifest(v, "", "S", "u")
        pu.build_manifest("2.1.1-test", "", "S", "u")

    def test_atomic_write(self):
        with tempfile.TemporaryDirectory() as d:
            p = pathlib.Path(d) / "sub" / "latest.json"
            pu.write_atomic(p, '{"a":1}')
            self.assertEqual(json.loads(p.read_text()), {"a": 1})
            pu.write_atomic(p, '{"a":2}')
            self.assertEqual(json.loads(p.read_text()), {"a": 2})
            self.assertEqual([x.name for x in p.parent.iterdir()], ["latest.json"])

    def test_refuses_key_dir_with_open_permissions(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            (d / "updater.key").write_text("k"); (d / "key.password").write_text("p")
            (d / "updater.key").chmod(0o644); (d / "key.password").chmod(0o600)
            with self.assertRaises(SystemExit):
                pu.sign(d / "x.exe", d)


if __name__ == "__main__":
    unittest.main()
