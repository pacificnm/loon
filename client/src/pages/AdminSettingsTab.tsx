import { useEffect, useState } from 'react';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { fetchHealth } from '../api/client';
import type { HealthResponse } from '../api/types';
import { FocusButton } from '../components/FocusButton';
import { getServerUrlOrNull, setServerUrl } from '../config';
import settingsStyles from './MovieEditPage.module.css';
import adminStyles from './AdminPage.module.css';

function ServerUrlInput({
  value,
  onChange,
  disabled,
}: {
  value: string;
  onChange: (value: string) => void;
  disabled: boolean;
}) {
  const { ref, focused } = useFocusable({
    focusKey: 'admin-settings-url',
    focusable: !disabled,
  });

  return (
    <input
      ref={ref}
      id="admin-settings-url"
      className={`${settingsStyles.input} ${focused ? settingsStyles.inputFocused : ''}`}
      type="text"
      inputMode="url"
      placeholder="http://192.168.88.10:3000"
      value={value}
      disabled={disabled}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

export function AdminSettingsTab() {
  const [serverUrl, setServerUrlInput] = useState(() => getServerUrlOrNull() ?? '');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [health, setHealth] = useState<HealthResponse | null>(null);

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'admin-settings-page',
    preferredChildFocusKey: 'admin-settings-url',
  });

  useEffect(() => {
    focusSelf();
  }, [focusSelf]);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setHealth(null);
    try {
      const saved = setServerUrl(serverUrl);
      const status = await fetchHealth(saved);
      setHealth(status);
      setServerUrlInput(saved);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save server URL');
    } finally {
      setSaving(false);
    }
  };

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} className={`${adminStyles.panel} ${settingsStyles.panel}`}>
        <label className={settingsStyles.label} htmlFor="admin-settings-url">
          Loon server URL
        </label>
        <p className={settingsStyles.hint}>
          Base URL for loon-server on your LAN (no trailing path). Saved on this TV and used
          instead of the build-time default.
        </p>
        <ServerUrlInput value={serverUrl} onChange={setServerUrlInput} disabled={saving} />

        {error ? <p className={settingsStyles.error}>{error}</p> : null}

        {health ? (
          <div className={adminStyles.statusBar}>
            <span>
              <span className={adminStyles.statusLabel}>Status: </span>
              <span className={adminStyles.statusValue}>{health.status}</span>
            </span>
            {health.version ? (
              <span>
                <span className={adminStyles.statusLabel}>Version: </span>
                <span className={adminStyles.statusValue}>{health.version}</span>
              </span>
            ) : null}
            <span>
              <span className={adminStyles.statusLabel}>Movies: </span>
              <span className={adminStyles.statusValue}>{health.movies_count}</span>
            </span>
          </div>
        ) : null}

        <div className={settingsStyles.actions}>
          <FocusButton
            focusKey="admin-settings-save"
            label={saving ? 'Saving…' : 'Test & Save'}
            onPress={() => void handleSave()}
          />
        </div>
      </div>
    </FocusContext.Provider>
  );
}
