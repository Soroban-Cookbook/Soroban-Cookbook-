"use client";

import { useState } from "react";
import { Bell, Mail, Smartphone, Globe } from "lucide-react";

const NOTIFICATION_TYPES = [
  { id: "tx_confirm", name: "TRANSACTION_CONFIRMATION", desc: "RECEIVE ALERTS WHEN CONTRACT CALLS ARE FINALIZED.", icon: Globe },
  { id: "security", name: "SECURITY_ALERTS", desc: "NOTIFICATIONS FOR NEW LOGIN COORDINATES OR PASSWORD CHANGES.", icon: Smartphone },
  { id: "marketing", name: "ECOSYSTEM_UPDATES", desc: "STAY INFORMED ABOUT NEW SMART CONTRACT TEMPLATES.", icon: Mail },
];

export default function NotificationsPage() {
  const [prefs, setPrefs] = useState({
    tx_confirm: true,
    security: true,
    marketing: false,
  });

  const toggle = (id: string) => {
    setPrefs(prev => ({ ...prev, [id as keyof typeof prev]: !prev[id as keyof typeof prev] }));
  };

  return (
    <div style={{ maxWidth: '700px' }}>
      <h2 className="glow-text">NOTIFICATION_CHANNELS</h2>

      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '30px' }}>
          <Bell size={20} />
          <h3 style={{ margin: 0 }}>ALERT_PREFERENCES</h3>
        </div>

        {NOTIFICATION_TYPES.map((type) => (
          <div key={type.id} style={{ 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            padding: '20px',
            borderBottom: '1px solid #111',
            gap: '20px'
          }}>
            <div style={{ display: 'flex', gap: '15px' }}>
              <type.icon size={24} color="#1d771d" style={{ marginTop: '4px' }} />
              <div>
                <div style={{ fontWeight: 'bold', fontSize: '1rem', color: '#33ff33' }}>{type.name}</div>
                <div style={{ fontSize: '0.7rem', color: '#1d771d', marginTop: '4px', lineHeight: '1.4' }}>{type.desc}</div>
              </div>
            </div>
            <button 
              onClick={() => toggle(type.id)}
              style={{ 
                minWidth: '100px',
                borderColor: prefs[type.id as keyof typeof prefs] ? '#33ff33' : '#333',
                color: prefs[type.id as keyof typeof prefs] ? '#33ff33' : '#333'
              }}
            >
              {prefs[type.id as keyof typeof prefs] ? "[ ENABLED ]" : "[ DISABLED ]"}
            </button>
          </div>
        ))}
      </div>

      <div style={{ marginTop: '30px', textAlign: 'right' }}>
        <button className="glow-text">SAVE_NOTIFICATION_PROFILE</button>
      </div>
    </div>
  );
}
