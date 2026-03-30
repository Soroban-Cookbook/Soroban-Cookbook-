import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Soroban Cookbook | Terminal Settings",
  description: "Manage your Soroban development environment settings.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>💾</text></svg>" />
      </head>
      <body>
        <header style={{ padding: '20px', borderBottom: '1px solid #222', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ fontWeight: 'bold' }}>SOROBAN COOKBOOK [SYSTEM_v8.0]</div>
          <div style={{ fontSize: '0.8rem' }}>SESSION: ACTIVE | NETWORK: FUTURENET</div>
        </header>
        <main>{children}</main>
        <footer style={{ padding: '10px 20px', borderTop: '1px solid #222', fontSize: '0.8rem', color: '#1d771d' }}>
          RUNNING AT: C:\USERS\SOROBAN\ROOT > _
        </footer>
      </body>
    </html>
  );
}
