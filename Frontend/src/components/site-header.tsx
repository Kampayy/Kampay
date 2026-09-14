import Link from "next/link";

import { Logo } from "@/components/logo";

const NAV = [
  { href: "#how-it-works", label: "How it works" },
  { href: "#products", label: "Products" },
  { href: "#credit", label: "Credit" },
  { href: "#stellar", label: "Why Stellar" },
];

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 border-b border-line bg-canvas/85 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
        <Link href="/" aria-label="KamPay home">
          <Logo />
        </Link>

        <nav aria-label="Main" className="hidden items-center gap-7 md:flex">
          {NAV.map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="text-sm text-muted transition-colors hover:text-ink"
            >
              {item.label}
            </a>
          ))}
        </nav>

        <div className="flex items-center gap-2">
          <a
            href="https://github.com/Kampayy/Kampay"
            className="hidden rounded-md px-3 py-2 text-sm text-muted transition-colors hover:text-ink sm:block"
          >
            GitHub
          </a>
          <a
            href="#waitlist"
            className="rounded-md bg-brand px-3.5 py-2 text-sm font-medium text-brand-ink transition-colors hover:bg-brand-hover"
          >
            Get early access
          </a>
        </div>
      </div>
    </header>
  );
}
