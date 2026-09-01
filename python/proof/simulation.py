from dataclasses import dataclass, field
from decimal import Decimal
from typing import List

@dataclass
class RegulatoryFlag:
    rule:        str
    severity:    str  # "info" | "review" | "block"
    description: str
    action:      str

@dataclass
class SimulationReport:
    spec_name:         str
    old_version:       str
    new_version:       str
    customers_total:   int
    customers_worse:   int
    customers_better:  int
    customers_neutral: int
    monthly_delta:     Decimal
    regulatory_flags:  List[RegulatoryFlag] = field(default_factory=list)
    verdict:           str = "deploy_clean"  # | "deploy_with_review" | "do_not_deploy"

class Simulation:
    """Phase 6 deliverable."""
    def run(self, old_spec, new_spec, portfolio_path: str) -> SimulationReport:
        raise NotImplementedError("Phase 6")
