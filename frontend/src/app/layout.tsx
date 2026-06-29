import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Soroban Cookbook",
  description: "Soroban smart contract cookbook.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
