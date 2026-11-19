import { Badge } from "../components/ui";
import type { InvestigationStatus } from "../api";

export const STATUS_LABELS: Record<InvestigationStatus, string> = {
  active: "Active",
  archived: "Archivée",
};

export function StatusBadge({ status }: { status: InvestigationStatus }) {
  return <Badge tone={status === "active" ? "success" : "neutral"}>{STATUS_LABELS[status]}</Badge>;
}
