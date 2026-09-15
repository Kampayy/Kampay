import { Container, SectionHeading } from "@/components/section";

const REASONS = [
  {
    title: "Fees that make micro-payroll viable",
    body: "Per-second streaming and 500-person batch runs are economically absurd where gas costs dollars. Here they cost fractions of a cent.",
  },
  {
    title: "Real off-ramps",
    body: "Stellar Anchors give a contributor an actual path from USDC to local currency in their own bank account — the part most payroll crypto hand-waves.",
  },
  {
    title: "Native stablecoins",
    body: "Circle-issued USDC on Stellar, plus a deep pool of tokenised fiat, rather than a bridged wrapper.",
  },
  {
    title: "Soroban",
    body: "Rust and WASM, metered execution, explicit storage lifetimes, and a testing story good enough to write financial logic against.",
  },
];

export function StellarSection() {
  return (
    <section id="stellar" className="section-wash scroll-mt-20 border-b border-line bg-sunken">
      <Container className="py-16 sm:py-20">
        <div className="grid gap-10 lg:grid-cols-[0.9fr_1.1fr] lg:gap-16">
          <SectionHeading
            eyebrow="Why Stellar"
            title="One chain, on purpose"
            lede="KamPay is Stellar-only. That is a decision, not a limitation — a credit record scattered across five ecosystems is not a credit record."
          />

          <div className="grid gap-x-8 gap-y-7 sm:grid-cols-2">
            {REASONS.map((reason) => (
              <div key={reason.title}>
                <h3 className="text-base font-semibold text-ink">
                  {reason.title}
                </h3>
                <p className="mt-1.5 text-sm leading-relaxed text-muted">
                  {reason.body}
                </p>
              </div>
            ))}
          </div>
        </div>
      </Container>
    </section>
  );
}
