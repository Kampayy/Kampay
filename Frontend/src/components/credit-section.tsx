import { Container, SectionHeading } from "@/components/section";

/** Mirrors the weights in `contract/contracts/credit/src/scoring.rs`. */
const WEIGHTS = [
  {
    label: "Payment history",
    weight: 35,
    detail: "Settled on time, without dispute",
  },
  { label: "Tenure", weight: 20, detail: "Age of your first settled payment" },
  {
    label: "Income consistency",
    weight: 15,
    detail: "Whether income actually recurs, or arrived in one burst",
  },
  {
    label: "Counterparty quality",
    weight: 10,
    detail: "How many distinct partners, and how they themselves rate",
  },
  {
    label: "Dispute record",
    weight: 10,
    detail: "Disputes lost — being disputed and vindicated costs nothing",
  },
  {
    label: "Loan repayment",
    weight: 10,
    detail: "Prior KamPay loans repaid on time",
  },
];

const PROPERTIES = [
  {
    title: "Portable",
    body: "The record lives on Stellar, not in a KamPay database. Any protocol can read it, and you keep it if you leave.",
  },
  {
    title: "Recomputable",
    body: "Every input is a public event and every weight is a constant in the contract. Nobody has to trust the number — it can be derived from scratch.",
  },
  {
    title: "Two-sided",
    body: "Employers are scored on whether they pay on time. A contributor can check a company's record before signing anything.",
  },
  {
    title: "Hard to fake",
    body: "Diversity of counterparties is half the graph score, so a closed loop of accounts paying each other caps out low. Volume alone is not an input at all.",
  },
];

export function CreditSection() {
  return (
    <section id="credit" className="scroll-mt-20 border-b border-line">
      <Container className="py-16 sm:py-20">
        <SectionHeading
          eyebrow="Credit"
          title="Three years of getting paid on time should be worth something"
          lede="A contractor with a spotless payment history has real creditworthiness and no way to prove it. KamPay writes that history to a ledger anyone can verify, and lenders can price."
        />

        <div className="mt-12 grid gap-10 lg:grid-cols-[1fr_1fr] lg:gap-14">
          <div>
            <h3 className="text-sm font-semibold text-ink">
              What the score is made of
            </h3>
            <ul className="mt-5 space-y-5">
              {WEIGHTS.map((item) => (
                <li key={item.label}>
                  <div className="flex items-baseline justify-between gap-4">
                    <span className="text-sm font-medium text-ink">
                      {item.label}
                    </span>
                    <span className="font-mono text-sm text-brand tabular-nums">
                      {item.weight}%
                    </span>
                  </div>
                  <div className="mt-2 h-1.5 rounded-full bg-sunken">
                    <div
                      className="h-1.5 rounded-full bg-brand"
                      style={{ width: `${(item.weight / 35) * 100}%` }}
                    />
                  </div>
                  <p className="mt-1.5 text-sm text-muted">{item.detail}</p>
                </li>
              ))}
            </ul>
          </div>

          <div>
            <h3 className="text-sm font-semibold text-ink">
              Why this one is different
            </h3>
            <div className="mt-5 grid gap-px overflow-hidden rounded-xl border border-line bg-line">
              {PROPERTIES.map((property) => (
                <div key={property.title} className="bg-surface p-5">
                  <h4 className="text-base font-semibold text-ink">
                    {property.title}
                  </h4>
                  <p className="mt-1.5 text-sm leading-relaxed text-muted">
                    {property.body}
                  </p>
                </div>
              ))}
            </div>

            <p className="mt-5 rounded-lg border border-line bg-sunken p-4 text-sm leading-relaxed text-muted">
              <span className="font-medium text-ink">On the limits.</span>{" "}
              Graph-based scoring is a brake on sybil attacks, not a wall. A
              well-funded attacker can open several accounts and pay themselves.
              What that costs them is real capital in motion and counterparties
              whose own standing never rises — so the ceiling is mediocre. The
              constraint is documented in the contract rather than glossed over.
            </p>
          </div>
        </div>
      </Container>
    </section>
  );
}
