<script lang="ts">
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { api, errMsg } from '../api';

  let { notify }: { notify: (m: string) => void } = $props();

  let profiles = $state<string[]>([]);
  let newName = $state('');
  let selected = $state('');

  const JSON_FILTER = [{ name: 'Perfil OpenF75', extensions: ['json'] }];

  async function refresh() {
    profiles = await api.listProfiles();
    if (!profiles.includes(selected)) selected = profiles[0] ?? '';
  }

  onMount(() => refresh().catch((e) => notify(errMsg(e))));

  async function run(fn: () => Promise<unknown>, ok: string) {
    try {
      await fn();
      await refresh();
      if (ok) notify(ok);
    } catch (e) {
      notify(errMsg(e));
    }
  }

  const saveCurrent = () => run(() => api.saveProfile(newName.trim()), 'Perfil salvo').then(() => (newName = ''));
  const apply = () => run(() => api.applyProfile(selected), 'Perfil aplicado');
  const remove = () => run(() => api.deleteProfile(selected), 'Perfil excluído');
  const backup = () => run(async () => notify(`Backup em ${await api.backupNow()}`), '');
  const restore = async () => {
    const path = await open({ filters: JSON_FILTER, multiple: false });
    if (typeof path !== 'string') return;
    await run(() => api.restoreBackup(path), 'Backup restaurado');
  };

  async function exportOne() {
    if (!selected) return;
    const path = await save({ defaultPath: `${selected}.json`, filters: JSON_FILTER });
    if (!path) return;
    await run(() => api.exportProfile(selected, path), 'Perfil exportado');
  }

  async function importOne() {
    const path = await open({ filters: JSON_FILTER, multiple: false });
    if (typeof path !== 'string') return;
    await run(async () => {
      const name = await api.importProfile(path);
      selected = name;
    }, 'Perfil importado');
  }
</script>

<section class="wrap">
  <div class="card">
    <h3>Perfis</h3>
    <div class="row">
      <select bind:value={selected} disabled={!profiles.length}>
        {#each profiles as p (p)}
          <option value={p}>{p}</option>
        {/each}
      </select>
      <button class="primary" onclick={apply} disabled={!selected}>Aplicar</button>
      <button onclick={remove} disabled={!selected}>Excluir</button>
      <button onclick={exportOne} disabled={!selected}>Exportar</button>
    </div>
    <div class="row">
      <input placeholder="Nome do novo perfil" bind:value={newName} />
      <button onclick={saveCurrent} disabled={!newName.trim()}>Salvar configuração atual</button>
    </div>
    <div class="row">
      <button onclick={importOne}>Importar perfil…</button>
    </div>
  </div>

  <div class="card">
    <h3>Backup do teclado</h3>
    <p class="hint">Um backup automático é criado na primeira conexão. Estes botões criam ou restauram um backup manual.</p>
    <div class="row">
      <button onclick={backup}>Fazer backup agora</button>
      <button onclick={restore}>Restaurar backup…</button>
    </div>
  </div>
</section>

<style>
  .wrap { display: grid; gap: 14px; max-width: 640px; }
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 14px; display: grid; gap: 10px; }
  .card h3 { margin: 0; font-size: 15px; }
  .row { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
  .row input { flex: 1; min-width: 160px; }
  .hint { margin: 0; color: var(--muted); font-size: 13px; }
</style>
