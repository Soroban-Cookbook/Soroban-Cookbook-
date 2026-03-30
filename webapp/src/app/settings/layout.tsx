"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { User, Settings, Key, Bell, Shield, Database } from "lucide-react";

const tabs = [
  { name: "Account", href: "/settings/account", icon: User },
  { name: "Preferences", href: "/settings/preferences", icon: Settings },
  { name: "API Keys", href: "/settings/api-keys", icon: Key },
  { name: "Notifications", href: "/settings/notifications", icon: Bell },
  { name: "Data & Privacy", href: "/settings/privacy", icon: Database },
];

export default function SettingsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const pathname = usePathname();

  return (
    <div style={{ display: 'flex', minHeight: 'calc(100vh - 140px)', border: '1px solid #222' }}>
      {/* Sidebar Navigation */}
      <nav style={{ width: '250px', borderRight: '1px solid #222', padding: '20px', backgroundColor: 'rgba(51, 255, 51, 0.01)' }}>
        <div style={{ marginBottom: '30px', color: '#1d771d', fontSize: '0.7rem' }}>
          [ MENU_NAVIGATION ]
        </div>
        <ul style={{ listStyle: 'none' }}>
          {tabs.map((tab) => {
            const isActive = pathname === tab.href;
            return (
              <li key={tab.name} style={{ marginBottom: '10px' }}>
                <Link 
                  href={tab.href}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '10px',
                    padding: '10px',
                    color: isActive ? '#33ff33' : '#1d771d',
                    backgroundColor: isActive ? 'rgba(51, 255, 51, 0.05)' : 'transparent',
                    border: '1px solid',
                    borderColor: isActive ? '#33ff33' : 'transparent',
                    textTransform: 'uppercase',
                    fontSize: '0.9rem',
                    transition: 'all 0.2s'
                  }}
                >
                  <tab.icon size={16} />
                  <span>{tab.name}</span>
                  {isActive && <span style={{ marginLeft: 'auto' }}>&gt;</span>}
                </Link>
              </li>
            );
          })}
        </ul>
        <div style={{ marginTop: '50px', borderTop: '1px solid #222', paddingTop: '20px' }}>
          <div style={{ color: '#1d771d', fontSize: '0.6rem', lineHeight: '1.4' }}>
            SYSTEM STATUS: NOMINAL<br/>
            ENCRYPTION: AES-256<br/>
            UPTIME: 99.99%<br/>
            LOGGED_IN_AS: ROOT
          </div>
        </div>
      </nav>

      {/* Main Content Area */}
      <section style={{ flex: 1, padding: '40px', overflowY: 'auto', position: 'relative' }}>
        <div style={{ 
          position: 'absolute', 
          top: '10px', 
          right: '20px', 
          fontSize: '0.7rem', 
          color: '#1d771d',
          zIndex: 10
        }}>
          PATH: {pathname.toUpperCase()}
        </div>
        {children}
      </section>
    </div>
  );
}
