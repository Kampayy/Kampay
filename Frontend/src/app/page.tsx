import { ClosingCta } from "@/components/closing-cta";
import { CreditSection } from "@/components/credit-section";
import { Hero } from "@/components/hero";
import { HowItWorks } from "@/components/how-it-works";
import { Products } from "@/components/products";
import { ProofStrip } from "@/components/proof-strip";
import { StellarSection } from "@/components/stellar-section";

export default function HomePage() {
  return (
    <>
      <Hero />
      <ProofStrip />
      <HowItWorks />
      <Products />
      <CreditSection />
      <StellarSection />
      <ClosingCta />
    </>
  );
}
