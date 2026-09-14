import { Container } from "@/components/section";

const FACTS = [
  { value: "3–5s", label: "Settlement finality" },
  { value: "<$0.01", label: "Network fee per payment" },
  { value: "1 tx", label: "To pay an entire team" },
  { value: "0%", label: "Collateral on score-backed loans" },
];

export function ProofStrip() {
  return (
    <section className="border-b border-line bg-sunken">
      <Container className="py-8">
        <dl className="grid grid-cols-2 gap-x-6 gap-y-7 lg:grid-cols-4">
          {FACTS.map((fact) => (
            <div key={fact.label}>
              <dt className="sr-only">{fact.label}</dt>
              <dd>
                <span className="block font-mono text-2xl font-semibold tracking-tight text-ink tabular-nums">
                  {fact.value}
                </span>
                <span className="mt-1 block text-sm text-muted">
                  {fact.label}
                </span>
              </dd>
            </div>
          ))}
        </dl>
      </Container>
    </section>
  );
}
