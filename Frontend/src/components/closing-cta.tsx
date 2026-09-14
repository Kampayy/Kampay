import { Container } from "@/components/section";

export function ClosingCta() {
  return (
    <section id="waitlist" className="scroll-mt-20">
      <Container className="py-16 sm:py-24">
        <div className="rounded-2xl border border-line bg-surface px-6 py-12 text-center sm:px-12">
          <h2 className="mx-auto max-w-2xl text-3xl font-semibold tracking-tight text-balance text-ink sm:text-4xl">
            Pay your team properly. Build them credit while you do it.
          </h2>
          <p className="mx-auto mt-4 max-w-xl text-base leading-relaxed text-pretty text-muted">
            The contracts are written and tested on Soroban. We are looking for
            companies and contributors to run the first payroll cycles on
            testnet.
          </p>

          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <a
              href="https://github.com/Kampayy/Kampay"
              className="rounded-md bg-brand px-5 py-2.5 text-sm font-medium text-brand-ink transition-colors hover:bg-brand-hover"
            >
              Read the contracts
            </a>
            <a
              href="https://github.com/Kampayy/Kampay/issues/new"
              className="rounded-md border border-line-strong px-5 py-2.5 text-sm font-medium text-ink transition-colors hover:bg-sunken"
            >
              Open an issue
            </a>
          </div>

          <p className="mt-6 text-sm text-faint">
            Pre-audit software. Testnet only — do not use with production funds.
          </p>
        </div>
      </Container>
    </section>
  );
}
