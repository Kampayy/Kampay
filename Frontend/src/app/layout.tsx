import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";

import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: {
    default: "KamPay — Payroll, escrow and on-chain credit on Stellar",
    template: "%s — KamPay",
  },
  description:
    "Lock funds upfront and pay contributors automatically. Recurring payroll, milestone escrow, and a portable on-chain credit score that unlocks under-collateralised loans — built on Stellar with Soroban.",
  keywords: [
    "payroll",
    "escrow",
    "credit score",
    "lending",
    "Stellar",
    "Soroban",
    "USDC",
  ],
  openGraph: {
    title: "KamPay — Payroll, escrow and on-chain credit on Stellar",
    description:
      "Programmable payroll and escrow that builds a portable credit score contributors can borrow against.",
    type: "website",
  },
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col font-sans">
        <a
          href="#main"
          className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:rounded-md focus:bg-brand focus:px-4 focus:py-2 focus:text-brand-ink"
        >
          Skip to content
        </a>
        <SiteHeader />
        <main id="main" className="flex-1">
          {children}
        </main>
        <SiteFooter />
      </body>
    </html>
  );
}
