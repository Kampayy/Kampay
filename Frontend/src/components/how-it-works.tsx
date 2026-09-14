import { Container, SectionHeading } from "@/components/section";

const STEPS = [
  {
    title: "Fund",
    body: "An employer deposits USDC into a vault. The balance is public, so a contributor can verify the money exists before starting work.",
  },
  {
    title: "Schedule",
    body: "Payroll streams accrue per second on a set cadence. Escrow contracts hold the full contract value against named milestones.",
  },
  {
    title: "Settle",
    body: "Payments execute automatically. Nobody has to chase an invoice or approve the happy path by hand.",
  },
  {
    title: "Score",
    body: "Each settled payment updates both parties' credit records — contributors and the companies paying them.",
  },
  {
    title: "Borrow",
    body: "A strong record unlocks loans with no collateral posted, repaid straight out of future payroll.",
  },
];

export function HowItWorks() {
  return (
    <section id="how-it-works" className="scroll-mt-20 border-b border-line">
      <Container className="py-16 sm:py-20">
        <SectionHeading
          eyebrow="How it works"
          title="Five steps, and the last two are the point"
          lede="Locked funds and automatic release solve the trust problem. What happens afterwards is what makes the history worth something."
        />

        <ol className="mt-12 grid gap-px overflow-hidden rounded-xl border border-line bg-line sm:grid-cols-2 lg:grid-cols-5">
          {STEPS.map((step, index) => (
            <li key={step.title} className="bg-surface p-5">
              <span className="font-mono text-xs text-faint tabular-nums">
                {String(index + 1).padStart(2, "0")}
              </span>
              <h3 className="mt-2 text-base font-semibold text-ink">
                {step.title}
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {step.body}
              </p>
            </li>
          ))}
        </ol>
      </Container>
    </section>
  );
}
