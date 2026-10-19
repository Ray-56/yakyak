"""Regression tests for document tooling; no SIP or media claims."""
from __future__ import annotations

import contextlib
import copy
import io
import json
from pathlib import Path
import tempfile
import unittest

import roadmap

SOURCE = Path(__file__).resolve().parents[1]


class RoadmapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        base = self.root / "docs" / "roadmap"
        base.mkdir(parents=True)
        for name in ("targets.json", "progress.json", "SCOPE.md", "TESTING_CONTRACT.md", "SOURCES.md"):
            (base / name).write_bytes((SOURCE / "docs" / "roadmap" / name).read_bytes())
        self.spec, self.progress = roadmap.load(self.root)
        # Tests are intentionally independent of future real progress.
        for record in self.progress["objectives"].values():
            record.update(status="not_started", implementation_commit=None, evidence=[], reviewer=None, reviewed_at=None, note="")
        (base / "progress.json").write_text(
            json.dumps(self.progress, ensure_ascii=False) + "\n", encoding="utf-8"
        )

    def report(self, ac_id: str = "AC-00-01") -> dict:
        # Synthetic temp test data, never written to the real progress file.
        record = self.progress["objectives"][ac_id]
        commit = "1" * 40
        relative = f"docs/acceptance/ch{ac_id[3:5]}/{ac_id}.md"
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"# Temporary test fixture {ac_id}\nTested commit: {commit}\n", encoding="utf-8")
        record.update(status="ready_for_review", implementation_commit=commit, evidence=[relative])
        return record

    def errors(self) -> list[str]:
        return roadmap.validate(self.root, self.spec, self.progress)

    def test_all_pending_data_valid(self) -> None:
        self.assertEqual([], self.errors())
        self.assertEqual(27, len(self.spec["chapters"]))
        self.assertEqual(125, len(self.progress["objectives"]))

    def test_duplicate_ac_rejected(self) -> None:
        self.spec["chapters"][0]["objectives"].append(copy.deepcopy(self.spec["chapters"][0]["objectives"][0]))
        self.assertTrue(any("duplicate objective" in e for e in self.errors()))

    def test_unknown_progress_id_rejected(self) -> None:
        self.progress["objectives"]["AC-99-99"] = {}
        self.assertTrue(any("unknown IDs" in e for e in self.errors()))

    def test_missing_progress_id_rejected(self) -> None:
        self.progress["objectives"].pop("AC-00-01")
        self.assertTrue(any("missing IDs" in e for e in self.errors()))

    def test_wrong_count_rejected(self) -> None:
        self.spec["total_objectives"] = 126
        self.assertTrue(any("total_objectives mismatch" in e for e in self.errors()))

    def test_cycle_rejected(self) -> None:
        self.spec["chapters"][0]["dependencies"] = ["01"]
        self.assertTrue(any("cycle" in e for e in self.errors()))

    def test_status_must_not_be_in_spec(self) -> None:
        self.spec["chapters"][0]["objectives"][0]["status"] = "passed"
        self.assertTrue(any("belongs in progress.json" in e for e in self.errors()))

    def test_unknown_status_rejected(self) -> None:
        self.progress["objectives"]["AC-00-01"]["status"] = "nearly_done"
        self.assertTrue(any("unknown status" in e for e in self.errors()))

    def test_blocked_needs_reason(self) -> None:
        self.progress["objectives"]["AC-00-01"]["status"] = "blocked"
        self.assertTrue(any("blocked requires" in e for e in self.errors()))

    def test_passed_needs_commit_report_and_review(self) -> None:
        self.progress["objectives"]["AC-00-01"]["status"] = "passed"
        errors = self.errors()
        for expected in ("implementation commit", "acceptance report", "maintainer review", "reviewed_at"):
            self.assertTrue(any(expected in error for error in errors), expected)

    def test_ready_report_structure_accepted(self) -> None:
        self.report()
        self.assertEqual([], self.errors())

    def test_passed_report_requires_review(self) -> None:
        record = self.report()
        record["status"] = "passed"
        self.assertTrue(any("maintainer review" in e for e in self.errors()))
        record.update(reviewer="synthetic-test-reviewer", reviewed_at="2026-10-16")
        self.assertEqual([], self.errors())

    def test_escape_report_rejected(self) -> None:
        record = self.report()
        record["evidence"] = ["../private.md"]
        self.assertTrue(any("report must exist beneath" in e for e in self.errors()))

    def test_external_chapter_symlink_rejected(self) -> None:
        record = self.report()
        report = self.root / record["evidence"][0]
        with tempfile.TemporaryDirectory() as outside:
            (Path(outside) / report.name).write_bytes(report.read_bytes())
            report.unlink()
            report.parent.rmdir()
            report.parent.symlink_to(outside, target_is_directory=True)
            self.assertTrue(any("report must exist beneath" in e for e in self.errors()))

    def test_wrong_chapter_report_rejected(self) -> None:
        record = self.report()
        record["evidence"] = ["docs/acceptance/ch01/AC-00-01.md"]
        self.assertTrue(any("report must exist beneath" in e for e in self.errors()))

    def test_report_contains_tested_commit(self) -> None:
        record = self.report()
        (self.root / record["evidence"][0]).write_text("# AC-00-01 no commit", encoding="utf-8")
        self.assertTrue(any("report must include" in e for e in self.errors()))

    def test_prerequisite_gate(self) -> None:
        record = self.report("AC-01-01")
        record.update(status="passed", reviewer="synthetic-test-reviewer", reviewed_at="2026-10-16")
        self.assertTrue(any("prerequisite chapter 00" in e for e in self.errors()))

    def test_render_is_deterministic_and_drift_detected(self) -> None:
        outputs = roadmap.views(self.root, self.spec, self.progress)
        self.assertEqual(30, len(outputs))
        self.assertEqual(outputs, roadmap.views(self.root, self.spec, self.progress))
        for relative, text in outputs.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
        self.assertEqual([], roadmap.check_views(self.root, outputs))
        target = self.root / "docs/roadmap/chapters/01.md"
        target.write_text(target.read_text(encoding="utf-8") + "drift", encoding="utf-8")
        before = target.read_bytes()
        self.assertTrue(any("drift" in e for e in roadmap.check_views(self.root, outputs)))
        self.assertEqual(before, target.read_bytes())

    def test_duplicate_json_keys_rejected(self) -> None:
        path = self.root / "duplicate.json"
        path.write_text('{"x":1,"x":2}', encoding="utf-8")
        with self.assertRaises(ValueError):
            roadmap.read_json(path)

    def test_unknown_show_returns_nonzero(self) -> None:
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(2, roadmap.main(["--root", str(self.root), "show", "AC-99-99"]))

    def test_ai_scope_cannot_be_accidentally_disabled(self) -> None:
        self.spec["ai_excluded"] = False
        self.assertTrue(any("ai_excluded" in e for e in self.errors()))


if __name__ == "__main__":
    unittest.main()
