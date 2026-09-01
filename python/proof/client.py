from .spec import Spec

class ProofClient:
    """
    High-level client wrapping the PROOF server API.
    Phase 5: connect to the running Axum server.
    """
    def __init__(self, base_url: str = "http://localhost:3001"):
        self.base_url = base_url

    def verify(self, spec: Spec, system_event: dict) -> dict:
        """Compare a system event against the spec. Phase 4."""
        raise NotImplementedError("Phase 4")

    def simulate(self, old_spec: Spec, new_spec: Spec, portfolio_path: str):
        """Replay a portfolio through old and new specs. Phase 6."""
        raise NotImplementedError("Phase 6")

    def audit(self, spec_name: str, export_format: str = None) -> dict:
        """Fetch or export the audit trail for a spec. Phase 8."""
        raise NotImplementedError("Phase 8")
