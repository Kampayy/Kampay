import { Container } from "@/components/section";
import { CreditCardPreview } from "@/components/credit-card-preview";

export function Hero() {
  return (
    <section className="relative overflow-hidden border-b border-line">
      <div
        className="grid-backdrop pointer-events-none absolute inset-0 opacity-60"
        aria-hidden="true"
      />

      <Container className="relative py-16 sm:py-24">
        <div className="grid items-center gap-12 lg:grid-cols-[1.05fr_0.95fr] lg:gap-16">
          <div>
            <p className="inline-flex items-center gap-2 rounded-full border border-line bg-surface px-3 py-1 text-xs text-muted">
              <span className="h-1.5 w-1.5 rounded-full bg-brand" />
              Built on Stellar with Soroban
            </p>

            <h1 className="mt-5 text-4xl font-semibold leading-[1.08] tracking-tight text-balance text-ink sm:text-5xl lg:text-[3.4rem]">
              Get paid on time. Then borrow against having been.
            </h1>

            <p className="mt-5 max-w-xl text-lg leading-relaxed text-pretty text-muted">
              KamPay locks payroll and contract funds upfront, releases them
              automatically, and turns every settled payment into a portable
              on-chain credit score — the kind a bank never gave you for three
              years of reliable freelance income.
            </p>

            <div className="mt-8 flex flex-wrap items-center gap-3">
              <a
                href="#waitlist"
                className="rounded-md bg-brand px-5 py-2.5 text-sm font-medium text-brand-ink transition-colors hover:bg-brand-hover"
              >
                Get early access
              </a>
              <a
                href="#how-it-works"
                className="rounded-md border border-line-strong px-5 py-2.5 text-sm font-medium text-ink transition-colors hover:bg-surface"
              >
                See how it works
              </a>
            </div>

            <p className="mt-5 text-sm text-faint">
              Open source. Non-custodial. Settles in 3–5 seconds.
            </p>
          </div>

          <div className="lg:pl-4">
            <CreditCardPreview />
          </div>
        </div>
      </Container>
    </section>
  );
}
