from dataclasses import dataclass
from pathlib import Path

@dataclass
class Spec:
    """A loaded PROOF specification."""
    path: Path
    name: str
    version: str
    raw: str

    @classmethod
    def load(cls, path: str) -> "Spec":
        p = Path(path)
        raw = p.read_text()
        # Phase 1: parse spec name and version from raw
        return cls(path=p, name=p.stem, version="0.1.0", raw=raw)

    def check(self, input: dict) -> dict:
        """Evaluate this spec against a single input. Phase 2."""
        raise NotImplementedError("Phase 2")
