import { Logo } from "@/components/logo";

const COLUMNS = [
  {
    heading: "Product",
    links: [
      { href: "#products", label: "Payroll streams" },
      { href: "#products", label: "Milestone escrow" },
      { href: "#credit", label: "Credit score" },
      { href: "#products", label: "Lending market" },
    ],
  },
  {
    heading: "Build",
    links: [
      { href: "https://github.com/Kampayy/Kampay", label: "GitHub" },
      {
        href: "https://github.com/Kampayy/Kampay/tree/main/contract",
        label: "Contracts",
      },
      {
        href: "https://developers.stellar.org/docs/build/smart-contracts/overview",
        label: "Soroban docs",
      },
    ],
  },
  {
    heading: "Network",
    links: [
      { href: "https://stellar.org", label: "Stellar" },
      {
        href: "https://developers.stellar.org/docs/build/apps/example-application-tutorial",
        label: "Anchors" ,
      },
      { href: "https://stellar.expert", label: "Explorer" },
    ],
  },
];

export function SiteFooter() {
  return (
    <footer className="border-t border-line bg-sunken">
      <div className="mx-auto max-w-6xl px-4 py-14 sm:px-6">
        <div className="grid gap-10 sm:grid-cols-2 lg:grid-cols-4">
          <div className="lg:pr-8">
            <Logo />
            <p className="mt-3 max-w-xs text-sm leading-relaxed text-muted">
              Programmable payroll, escrow, and on-chain credit. Built on
              Stellar with Soroban.
            </p>
          </div>

          {COLUMNS.map((column) => (
            <div key={column.heading}>
              <h2 className="text-xs font-semibold uppercase tracking-wider text-faint">
                {column.heading}
              </h2>
              <ul className="mt-3 space-y-2">
                {column.links.map((link) => (
                  <li key={link.label}>
                    <a
                      href={link.href}
                      className="text-sm text-muted transition-colors hover:text-ink"
                    >
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-12 flex flex-col gap-3 border-t border-line pt-6 text-sm text-faint sm:flex-row sm:items-center sm:justify-between">
          <p>© {new Date().getFullYear()} KamPay. MIT licensed.</p>
          <p className="text-danger">
            Pre-audit software — do not use with production funds.
          </p>
        </div>
      </div>
    </footer>
  );
}
