"use client";

import { useState } from "react";
import { Mail, Lock, ShieldCheck } from "lucide-react";

export default function AccountPage() {
  const [password, setPassword] = useState("");
  const [strength, setStrength] = useState(0);

  const calculateStrength = (pwd: string) => {
    let s = 0;
    if (pwd.length > 8) s++;
    if (/[A-Z]/.test(pwd)) s++;
    if (/[0-9]/.test(pwd)) s++;
    if (/[^A-Za-z0-9]/.test(pwd)) s++;
    setStrength(s);
  };

  const getStrengthLabel = () => {
    if (strength === 0) return "NONE";
    if (strength === 1) return "WEAK";
    if (strength === 2) return "MODERATE";
    if (strength === 3) return "STRONG";
    return "EXTREME";
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h2 className="glow-text">ACCOUNT_SETTINGS</h2>
      
      {/* Email Section */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <Mail size={20} />
          <h3 style={{ margin: 0 }}>EMAIL_ADDRESS</h3>
        </div>
        <span className="terminal-label">CURRENT_EMAIL</span>
        <div style={{ marginBottom: '20px', color: '#33ff33' }}>developer@soroban.cookbook</div>
        <span className="terminal-label">UPDATE_EMAIL</span>
        <input type="email" placeholder="ENTER NEW EMAIL..." />
        <button style={{ marginTop: '10px' }}>UPDATE_PRIMARY_EMAIL</button>
      </div>

      {/* Password Section */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <Lock size={20} />
          <h3 style={{ margin: 0 }}>PASSWORD_SECURITY</h3>
        </div>
        <span className="terminal-label">CURRENT_PASSWORD</span>
        <input type="password" placeholder="********" />
        <span className="terminal-label">NEW_PASSWORD</span>
        <input 
          type="password" 
          placeholder="ENTER NEW PASSWORD..." 
          value={password}
          onChange={(e) => {
            setPassword(e.target.value);
            calculateStrength(e.target.value);
          }}
        />
        <div style={{ marginBottom: '20px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.7rem', marginBottom: '5px' }}>
            <span>STRENGTH:</span>
            <span className={strength > 2 ? 'glow-text' : 'amber-text'}>{getStrengthLabel()}</span>
          </div>
          <div style={{ height: '4px', background: '#222', width: '100%', position: 'relative' }}>
            <div style={{ 
              height: '100%', 
              background: strength > 2 ? '#33ff33' : '#ffb000', 
              width: `${(strength / 4) * 100}%`,
              transition: 'width 0.3s ease'
            }}></div>
          </div>
        </div>
        <button>CHANGE_PASSWORD</button>
      </div>

      {/* 2FA Section */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <ShieldCheck size={20} />
          <h3 style={{ margin: 0 }}>AUTHENTICATION_2FA</h3>
        </div>
        <p style={{ fontSize: '0.8rem', color: '#1d771d', marginBottom: '20px', lineHeight: '1.4' }}>
          MULTI-FACTOR AUTHENTICATION ADDS AN EXTRA LAYER OF SECURITY TO YOUR GLOBAL COORDINATES.
        </p>
        <button className="amber-text" style={{ borderColor: '#ffb000' }}>ENABLE_TWO_FACTOR_AUTH</button>
      </div>
    </div>
  );
}
