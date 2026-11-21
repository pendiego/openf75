<script lang="ts">
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { api, errMsg } from '../api';
  import AsyncButton from '../AsyncButton.svelte';

  let { notify }: { notify: (m: string) => void } = $props();

  let profiles = $state<string[]>([]);
  let newName = $state('');

  const JSON_FILTER = [{ name: 'Perfil OpenF75', extensions: ['json'] }];

  async function refresh() {
    profiles = await api.listProfiles();
  }

  onMount(() => refresh().catch((e) => notify(errMsg(e))));

  // Throws on failure so AsyncButton can show it; the list is refreshed after every write.
  async function write(fn: () => Promise<unknown>, ok: string) {
    await fn();
    await refresh();
    if (ok) notify(ok);
  }

  async function run(fn: () => Promise<unknown>, ok: string) {
    try {
      await write(fn, ok);
    } catch (e) {
      notify(errMsg(e));
    }
  }

  async function saveCurrent() {
    await write(() => api.saveProfile(newName.trim()), 'Perfil salvo');
    newName = '';
  }
  const apply = (name: string) => write(() => api.applyProfile(name), 'Perfil aplicado');
  const remove = (name: string) => write(() => api.deleteProfile(name), 'Perfil excluído');
  const backup = () => write(async () => notify(`Backup em ${await api.backupNow()}`), '');
  const restore = async () => {
    const path = await open({ filters: JSON_FILTER, multiple: false });
    if (typeof path !== 'string') return;
    await run(() => api.restoreBackup(path), 'Backup restaurado');
  };

  async function exportOne(name: string) {
    const path = await save({ defaultPath: `${name}.json`, filters: JSON_FILTER });
    if (!path) return;
    await run(() => api.exportProfile(name, path), 'Perfil exportado');
  }

  async function importOne() {
    const path = await open({ filters: JSON_FILTER, multiple: false });
    if (typeof path !== 'string') return;
    await run(() => api.importProfile(path), 'Perfil importado');
  }
</script>

<div class="workspace">
  <div class="stage">
    <div class="stage-body">
      <div class="stack narrow">
        <div>
          <p class="label">Perfis salvos</p>
          {#if profiles.length}
            {#each profiles as p (p)}
              <div class="profile">
                <span class="profile-name">{p}</span>
                <div class="row">
                  <AsyncButton run={() => apply(p)} notify={notify} label="Aplicar" busyLabel="Aplicando…" />
                  <button class="ghost" onclick={() => exportOne(p)}>Exportar</button>
                  <AsyncButton class="ghost" run={() => remove(p)} notify={notify} label="Excluir" busyLabel="Excluindo…" />
                </div>
              </div>
            {/each}
          {:else}
            <p class="empty">Nenhum perfil salvo ainda. Salve a configuração atual ou importe um arquivo.</p>
          {/if}
        </div>

        <div>
          <p class="label">Novo perfil</p>
          <div class="setting">
            <h3>Salvar configuração atual</h3>
            <div class="row">
              <input aria-label="Nome do novo perfil" placeholder="Nome do novo perfil" bind:value={newName} />
              <AsyncButton class="primary" run={saveCurrent} notify={notify} label="Salvar" busyLabel="Salvando…" disabled={!newName.trim()} />
            </div>
            <p class="hint">Grava o estado atual do teclado como um perfil com este nome.</p>
          </div>
          <div class="setting">
            <h3>Importar perfil</h3>
            <button onclick={importOne}>Importar perfil…</button>
            <p class="hint">Carrega um arquivo de perfil exportado anteriormente.</p>
          </div>
        </div>

        <div>
          <p class="label">Backup</p>
          <div class="setting">
            <h3>Fazer backup agora</h3>
            <AsyncButton run={backup} notify={notify} label="Fazer backup agora" busyLabel="Fazendo backup…" />
            <p class="hint">Um backup automático é criado na primeira conexão. Este botão cria um backup manual.</p>
          </div>
          <div class="setting">
            <h3>Restaurar backup</h3>
            <button onclick={restore}>Restaurar backup…</button>
            <p class="hint">Restaura as configurações a partir de um arquivo de backup.</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .profile {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--line);
  }
  .profile-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .row input { flex: 1; min-width: 160px; }
</style>
