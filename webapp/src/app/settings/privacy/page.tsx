"use client";

import { Shield, Download, Trash2, AlertTriangle } from "lucide-react";

export default function PrivacyPage() {
  return (
    <div style={{ maxWidth: '700px' }}>
      <h2 className="glow-text">DATA_AND_PRIVACY_PROTOCOLS</h2>

      {/* Data Portability */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <Download size={20} />
          <h3 style={{ margin: 0 }}>DATA_PORTABILITY</h3>
        </div>
        <p style={{ fontSize: '0.8rem', color: '#1d771d', marginBottom: '20px', lineHeight: '1.4' }}>
          DOWNLOAD A COMPLETE ARCHIVE OF YOUR SYSTEM CONFIGURATIONS, API LOGS, AND SAVED PREFERENCES IN JSON FORMAT.
        </p>
        <button>INITIALIZE_DATA_EXPORT</button>
      </div>

      {/* Account Deletion */}
      <div className="terminal-card" style={{ borderColor: 'rgba(255, 51, 51, 0.3)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px', color: '#ff3333' }}>
          <Trash2 size={20} />
          <h3 style={{ margin: 0 }}>TERMINATE_ACCOUNT</h3>
        </div>
        <div style={{ 
          backgroundColor: 'rgba(255, 51, 51, 0.05)', 
          padding: '15px', 
          borderLeft: '4px solid #ff3333',
          marginBottom: '20px'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#ff3333', fontWeight: 'bold', fontSize: '0.8rem', marginBottom: '8px' }}>
            <AlertTriangle size={16} /> CRITICAL_WARNING
          </div>
          <p style={{ fontSize: '0.7rem', color: '#ff3333', lineHeight: '1.4' }}>
            THIS ACTION IS IRREVERSIBLE. ALL API KEYS WILL BE REVOKED, AND YOUR REPOSITORY ACCESS WILL BE TERMINATED. 
            ENSURE YOU HAVE BACKED UP ALL NECESSARY CONTRACT DATA.
          </p>
        </div>
        <button style={{ borderColor: '#ff3333', color: '#ff3333' }}>REQUEST_ACCOUNT_DELETION</button>
      </div>

      {/* Compliance Information */}
      <div style={{ marginTop: '40px', padding: '20px', border: '1px dashed #222' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '15px' }}>
          <Shield size={16} color="#1d771d" />
          <span style={{ fontSize: '0.7rem', color: '#1d771d', fontWeight: 'bold' }}>COMPLIANCE_PROTOCOL_v4.2</span>
        </div>
        <p style={{ fontSize: '0.65rem', color: '#1d771d', lineHeight: '1.6' }}>
          SOROBAN-COOKBOOK ADHERES TO GLOBAL DATA PROTECTION STANDARDS. YOUR ENCRYPTED PAYLOADS REMAIN 
          PRIVATE AND ARE NEVER STORED ON UNSECURED LEDGERS. FOR MORE INFORMATION ON OUR PRIVACY 
          WHITEPAPER, ACCESS THE DOCUMENTATION AT /DOCS/SECURITY.
        </p>
      </div>
    </div>
  );
}
