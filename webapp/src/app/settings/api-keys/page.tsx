"use client";

import { useState } from "react";
import { Plus, Trash2, Copy, Check } from "lucide-react";

const INITIAL_KEYS = [
  { id: "1", name: "DEVELOPMENT_KEY", key: "sk_test_51Mz9...x9J2", created: "2026-03-24" },
  { id: "2", name: "PRODUCTION_MAIN", key: "sk_live_82F1...aZ88", created: "2026-03-01" },
];

export default function ApiKeysPage() {
  const [keys, setKeys] = useState(INITIAL_KEYS);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const generateKey = () => {
    const newKey = {
      id: Math.random().toString(36).substr(2, 9),
      name: "NEW_GENERATED_KEY",
      key: `sk_test_${Math.random().toString(36).substr(2, 8)}...${Math.random().toString(36).substr(2, 4)}`,
      created: new Date().toISOString().split('T')[0],
    };
    setKeys([newKey, ...keys]);
  };

  const deleteKey = (id: string) => {
    setKeys(keys.filter(k => k.id !== id));
  };

  const copyToClipboard = (id: string, key: string) => {
    navigator.clipboard.writeText(key);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  return (
    <div style={{ maxWidth: '800px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
        <h2 className="glow-text">API_ACCESS_KEYS</h2>
        <button onClick={generateKey} style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Plus size={16} /> GENERATE_NEW
        </button>
      </div>

      <div className="terminal-card" style={{ padding: 0 }}>
        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: '2fr 3fr 1.5fr 1fr', 
          padding: '15px 20px', 
          borderBottom: '1px solid #222', 
          fontSize: '0.7rem', 
          color: '#1d771d' 
        }}>
          <div>IDENTIFIER</div>
          <div>KEY_TOKEN</div>
          <div>DATE_CREATED</div>
          <div style={{ textAlign: 'right' }}>ACTIONS</div>
        </div>

        {keys.map((k) => (
          <div key={k.id} style={{ 
            display: 'grid', 
            gridTemplateColumns: '2fr 3fr 1.5fr 1fr', 
            padding: '15px 20px', 
            borderBottom: '1px solid #111', 
            alignItems: 'center',
            fontSize: '0.85rem'
          }}>
            <div style={{ fontWeight: 'bold' }}>{k.name}</div>
            <div style={{ color: '#1d771d', fontFamily: 'monospace' }}>{k.key}</div>
            <div style={{ fontSize: '0.7rem' }}>{k.created}</div>
            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '15px' }}>
              <button 
                onClick={() => copyToClipboard(k.id, k.key)}
                style={{ padding: '4px', border: 'none', background: 'transparent' }}
              >
                {copiedId === k.id ? <Check size={16} color="#33ff33" /> : <Copy size={16} color="#1d771d" />}
              </button>
              <button 
                onClick={() => deleteKey(k.id)}
                style={{ padding: '4px', border: 'none', background: 'transparent' }}
              >
                <Trash2 size={16} color="#ff3333" />
              </button>
            </div>
          </div>
        ))}
      </div>

      <p style={{ marginTop: '20px', fontSize: '0.7rem', color: '#1d771d', lineHeight: '1.4' }}>
        WARNING: API KEYS PROVIDE FULL ACCESS TO YOUR SOROBAN CONTRACT RESOURCES. 
        NEVER SHARE YOUR SECRET KEYS OR COMMIT THEM TO VERSION CONTROL.
      </p>
    </div>
  );
}
