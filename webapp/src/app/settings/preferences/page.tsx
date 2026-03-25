"use client";

import { useId } from "react";
import { Globe, Palette, Languages } from "lucide-react";

export default function PreferencesPage() {
  const timezoneId = useId();
  const themeId = useId();
  const languageId = useId();

  return (
    <div style={{ maxWidth: '600px' }}>
      <h2 className="glow-text">SYSTEM_PREFERENCES</h2>

      {/* Timezone Selection */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <Globe size={20} />
          <h3 style={{ margin: 0 }}>TEMPORAL_ZONE</h3>
        </div>
        <span className="terminal-label">CURRENT_ZONE</span>
        <select id={timezoneId}>
          <option value="UTC">UTC (GMT+00:00)</option>
          <option value="EST">EST (GMT-05:00)</option>
          <option value="CET">CET (GMT+01:00)</option>
          <option value="IST">IST (GMT+05:30)</option>
          <option value="JST">JST (GMT+09:00)</option>
        </select>
        <p style={{ fontSize: '0.6rem', color: '#1d771d', marginTop: '5px' }}>
          * SYSTEM TIME WILL SYNCHRONIZE WITH SELECTED COORDINATES.
        </p>
      </div>

      {/* Theme Selection */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <Palette size={20} />
          <h3 style={{ margin: 0 }}>VISUAL_INTERFACE</h3>
        </div>
        <span className="terminal-label">ACTIVE_THEME</span>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '10px' }}>
          <button style={{ fontSize: '0.7rem' }}>MATRIX_GREEN</button>
          <button style={{ fontSize: '0.7rem', color: '#ffb000', borderColor: '#ffb000' }}>AMBER_VT100</button>
          <button style={{ fontSize: '0.7rem', color: '#fff', borderColor: '#222' }}>CLASSIC_MONO</button>
        </div>
      </div>

      {/* Language Selection */}
      <div className="terminal-card">
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '20px' }}>
          <Languages size={20} />
          <h3 style={{ margin: 0 }}>LINGUISTIC_CORE</h3>
        </div>
        <span className="terminal-label">SYSTEM_LANGUAGE</span>
        <select id={languageId}>
          <option value="en">ENGLISH (US-DEFAULT)</option>
          <option value="es">ESPAÑOL</option>
          <option value="fr">FRANÇAIS</option>
          <option value="de">DEUTSCH</option>
          <option value="ja">日本語</option>
        </select>
        <button style={{ marginTop: '20px' }}>SAVE_PREFERENCES</button>
      </div>
    </div>
  );
}
