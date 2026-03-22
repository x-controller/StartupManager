<template>
  <div class="app">
    <header class="header" data-tauri-drag-region>
      <div class="header-left">
        <div class="logo">⚡</div>
        <h1>Startup Manager <span>项目启动管理器</span></h1>
      </div>
      <div class="header-actions">
        <div class="search-box">
          <span class="search-icon">🔍</span>
          <input
            type="text"
            v-model="searchQuery"
            placeholder="搜索项目..."
            @input="filterProjects"
          />
        </div>
        <button class="btn btn-icon" :title="viewMode === 'card' ? '切换表格视图' : '切换卡片视图'" @click="toggleViewMode">
          {{ viewMode === 'card' ? '📋' : '🃏' }}
        </button>
        <button class="btn btn-icon" title="设置" @click="openSettingsModal">⚙️</button>
        <button class="btn btn-accent" @click="openAddModal">＋ 新建项目</button>
      </div>
    </header>

    <main class="main">
      <div class="stats-bar">
        <div class="stat-item">
          <div class="stat-dot total"></div>
          <span class="stat-value">{{ stats.total }}</span>
          <span class="stat-label">总项目</span>
        </div>
        <div class="stat-item">
          <div class="stat-dot running"></div>
          <span class="stat-value">{{ stats.running }}</span>
          <span class="stat-label">运行中</span>
        </div>
        <div class="stat-item">
          <div class="stat-dot stopped"></div>
          <span class="stat-value">{{ stats.stopped }}</span>
          <span class="stat-label">已停止</span>
        </div>
      </div>

      <!-- Card View -->
      <div v-if="viewMode === 'card'" class="projects-grid">
        <div
          v-if="filteredProjects.length === 0"
          class="empty-state"
          style="grid-column: 1/-1"
        >
          <div class="empty-icon">{{ projects.length === 0 ? '📦' : '🔍' }}</div>
          <h3>
            {{ projects.length === 0 ? '还没有项目' : '未找到匹配项目' }}
          </h3>
          <p>
            {{ projects.length === 0 ? '点击「新建项目」添加你的第一个启动项' : '尝试其他搜索关键词' }}
          </p>
          <button
            v-if="projects.length === 0"
            class="btn btn-accent"
            @click="openAddModal"
          >
            ＋ 新建项目
          </button>
        </div>

        <div
          v-for="project in filteredProjects"
          :key="project.id"
          class="project-card"
          :class="{ running: isRunning(project.id) }"
          :data-id="project.id"
          :style="`--card-color: ${project.color || '#4a6cf7'}`"
        >
          <div class="card-header">
            <div class="card-title-group">
              <div class="card-title">
                {{ project.name }}
                <span
                  class="status-badge"
                  :class="isRunning(project.id) ? 'running' : 'stopped'"
                >
                  {{ isRunning(project.id) ? '● 运行中' : '○ 已停止' }}
                </span>
              </div>
              <div v-if="project.description" class="card-desc">
                {{ project.description }}
              </div>
            </div>
            <div class="card-actions">
              <button
                class="btn btn-sm btn-icon"
                title="日志"
                @click="openLogModal(project.id)"
              >
                📋
              </button>
              <button
                class="btn btn-sm btn-icon"
                title="编辑"
                @click="openEditModal(project.id)"
              >
                ✏️
              </button>
              <button
                class="btn btn-sm btn-icon"
                title="删除"
                @click="openDeleteModal(project.id)"
              >
                🗑️
              </button>
            </div>
          </div>

          <div class="card-info">
            <div
              class="info-row"
              style="cursor: pointer"
              @click="openDirectory(project.id)"
              title="在资源管理器中打开"
            >
              <span class="info-icon">📁</span>
              <span style="text-decoration: underline dotted">
                {{ project.directory }}
              </span>
            </div>
            <div class="info-row">
              <span class="info-icon">▶</span>
              {{ project.command }}
            </div>
            <div v-if="project.port" class="info-row">
              <span class="info-icon">🔌</span>
              端口: {{ project.port }}
            </div>
            <div v-if="project.url" class="info-row">
              <span class="info-icon">🔗</span>
              <a
                href="javascript:void(0)"
                @click="openUrl(project.url)"
                style="color: var(--accent); text-decoration: none; cursor: pointer"
              >
                {{ project.url }}
              </a>
            </div>
            <div v-if="getPid(project.id)" class="info-row">
              <span class="info-icon">⚙</span>
              PID: {{ getPid(project.id) }}
            </div>
          </div>

          <div class="card-footer">
            <button
              class="btn btn-success start-btn"
              :style="{ display: isRunning(project.id) ? 'none' : 'flex' }"
              @click="startProject(project.id)"
            >
              ▶ 启动
            </button>
            <button
              class="btn btn-danger stop-btn"
              :style="{ display: isRunning(project.id) ? 'flex' : 'none' }"
              @click="stopProject(project.id)"
            >
              ■ 停止
            </button>
          </div>
        </div>
      </div>

      <!-- Table View -->
      <div v-if="viewMode === 'table'" class="projects-table-container">
        <table class="projects-table">
          <thead>
            <tr>
              <th>状态</th>
              <th>项目名称</th>
              <th>启动命令</th>
              <th>端口</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-if="filteredProjects.length === 0"
            >
              <td colspan="5" class="table-empty">
                <div class="empty-state">
                  <div class="empty-icon">{{ projects.length === 0 ? '📦' : '🔍' }}</div>
                  <h3>
                    {{ projects.length === 0 ? '还没有项目' : '未找到匹配项目' }}
                  </h3>
                  <p>
                    {{ projects.length === 0 ? '点击「新建项目」添加你的第一个启动项' : '尝试其他搜索关键词' }}
                  </p>
                  <button
                    v-if="projects.length === 0"
                    class="btn btn-accent"
                    @click="openAddModal"
                  >
                    ＋ 新建项目
                  </button>
                </div>
              </td>
            </tr>
            <tr
              v-for="project in filteredProjects"
              :key="project.id"
              :class="{ running: isRunning(project.id) }"
            >
              <td class="col-status">
                <span
                  class="status-badge"
                  :class="isRunning(project.id) ? 'running' : 'stopped'"
                >
                  {{ isRunning(project.id) ? '● 运行中' : '○ 已停止' }}
                </span>
              </td>
              <td class="col-name">
                <div class="table-name">{{ project.name }}</div>
                <div class="table-desc" v-if="project.description">{{ project.description }}</div>
              </td>
              <td class="col-command" title="{{ project.directory }}">
                <div class="table-command">{{ project.command }}</div>
                <div class="table-directory">{{ project.directory }}</div>
              </td>
              <td class="col-port">{{ project.port || '-' }}</td>
              <td class="col-actions">
                <button class="btn btn-sm" @click="openDirectory(project.id)" title="打开目录">
                  📁
                </button>
                <button class="btn btn-sm" @click="openLogModal(project.id)" title="日志">
                  📋
                </button>
                <button class="btn btn-sm" @click="openEditModal(project.id)" title="编辑">
                  ✏️
                </button>
                <button
                  v-if="!isRunning(project.id)"
                  class="btn btn-sm btn-success"
                  @click="startProject(project.id)"
                  title="启动"
                >
                  ▶
                </button>
                <button
                  v-if="isRunning(project.id)"
                  class="btn btn-sm btn-danger"
                  @click="stopProject(project.id)"
                  title="停止"
                >
                  ■
                </button>
                <button class="btn btn-sm btn-danger" @click="openDeleteModal(project.id)" title="删除">
                  🗑️
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </main>

    <!-- Add/Edit Modal -->
    <teleport to="body">
      <transition name="fade">
        <div
          v-if="projectModalVisible"
          class="modal-overlay active"
          @click.self="closeModal"
          @keydown.escape="closeModal"
        >
          <div class="modal">
            <div class="modal-header">
              <h2>{{ modalTitle }}</h2>
              <button class="btn btn-icon" @click="closeModal">&times;</button>
            </div>
            <div class="modal-body">
              <div class="form-group">
                <label>项目名称</label>
                <input
                  v-model="form.name"
                  type="text"
                  placeholder="例: 前端开发服务"
                />
              </div>
              <div class="form-group">
                <label>工作目录</label>
                <input
                  v-model="form.directory"
                  type="text"
                  placeholder="例: D:\code\my-project"
                />
                <span class="hint">项目所在的绝对路径</span>
              </div>
              <div class="form-group">
                <label>启动命令</label>
                <input
                  v-model="form.command"
                  type="text"
                  placeholder="例: npm run dev"
                />
                <span class="hint">在工作目录中执行的命令（无需 cd）</span>
              </div>
              <div class="form-group">
                <label>端口</label>
                <input
                  v-model="form.port"
                  type="text"
                  placeholder="例: 8080"
                />
                <span class="hint">项目运行的端口号（可选）</span>
              </div>
              <div class="form-group">
                <label>访问链接</label>
                <input
                  v-model="form.url"
                  type="text"
                  placeholder="例: http://localhost:8080"
                />
                <span class="hint">项目的访问地址（可选）</span>
              </div>
              <div class="form-group">
                <label>描述（可选）</label>
                <textarea
                  v-model="form.description"
                  placeholder="项目说明..."
                  rows="2"
                />
              </div>
              <div class="form-group">
                <label>标签颜色</label>
                <div class="color-picker">
                  <div
                    v-for="color in COLORS"
                    :key="color"
                    class="color-option"
                    :class="{ active: form.color === color }"
                    :style="{ background: color }"
                    @click="form.color = color"
                  />
                </div>
              </div>
            </div>
            <div class="modal-footer">
              <button class="btn" @click="closeModal">取消</button>
              <button class="btn btn-accent" @click="saveProject">保存</button>
            </div>
          </div>
        </div>
      </transition>
    </teleport>

    <!-- Delete Confirm Modal -->
    <teleport to="body">
      <transition name="fade">
        <div
          v-if="deleteModalVisible"
          class="modal-overlay active"
          @click.self="closeDeleteModal"
          @keydown.escape="closeDeleteModal"
        >
          <div class="modal" style="width: 400px">
            <div class="modal-header">
              <h2>确认删除</h2>
              <button class="btn btn-icon" @click="closeDeleteModal">&times;</button>
            </div>
            <div class="confirm-msg">
              <p>
                确定要删除项目 <span class="project-name">{{ getProjectName(deletingId) }}</span> 吗？
              </p>
              <p>此操作不可撤销，如果项目正在运行将被终止。</p>
            </div>
            <div class="modal-footer">
              <button class="btn" @click="closeDeleteModal">取消</button>
              <button class="btn btn-danger" @click="confirmDelete">删除</button>
            </div>
          </div>
        </div>
      </transition>
    </teleport>

    <!-- Log Modal -->
    <teleport to="body">
      <transition name="fade">
        <div
          v-if="logModalVisible"
          class="modal-overlay active"
          @click.self="closeLogModal"
          @keydown.escape="closeLogModal"
        >
          <div class="modal log-modal">
            <div class="modal-header">
              <h2>{{ logModalTitle }} - 运行日志</h2>
              <div style="display: flex; gap: 8px; align-items: center">
                <button class="btn btn-sm" @click="clearLogs">清空</button>
                <button class="btn btn-icon" @click="closeLogModal">&times;</button>
              </div>
            </div>
            <div class="modal-body" style="padding: 16px">
              <pre class="log-content" ref="logContentRef">
                <span v-if="!currentLogs" class="log-empty">暂无日志</span>
                {{ currentLogs }}
              </pre>
            </div>
          </div>
        </div>
      </transition>
    </teleport>

    <!-- Settings Modal -->
    <teleport to="body">
      <transition name="fade">
        <div
          v-if="settingsModalVisible"
          class="modal-overlay active"
          @click.self="closeSettingsModal"
          @keydown.escape="closeSettingsModal"
        >
          <div class="modal" style="width: 460px">
            <div class="modal-header">
              <h2>应用设置</h2>
              <button class="btn btn-icon" @click="closeSettingsModal">&times;</button>
            </div>
            <div class="modal-body">
              <div class="setting-item">
                <div class="setting-info">
                  <div class="setting-name">开机自动启动</div>
                  <div class="setting-desc">登录 Windows 后自动启动应用</div>
                </div>
                <label class="toggle">
                  <input
                    v-model="autoStartEnabled"
                    type="checkbox"
                    @change="saveAutoStart"
                  />
                  <span class="slider"></span>
                </label>
              </div>
            </div>
          </div>
        </div>
      </transition>
    </teleport>

    <!-- Toast Container -->
    <teleport to="body">
      <div class="toast-container">
        <transition
          v-for="toast in toasts"
          :key="toast.id"
          name="toast"
          @after-leave="removeToast(toast.id)"
        >
          <div class="toast" :class="toast.type">
            <span>{{ toastIcon(toast.type) }}</span>
            <span>{{ toast.msg }}</span>
          </div>
        </transition>
      </div>
    </teleport>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke, open } from './composables/useTauri'
import { useToast } from './composables/useToast'

const { toasts, showToast } = useToast()

// Types
const COLORS = ['#4a6cf7', '#7c3aed', '#ec4899', '#ef4444', '#f59e0b', '#22c55e', '#06b6d4', '#8b5cf6']

// State
const projects = ref([])
const statuses = ref({})
const searchQuery = ref('')
const filteredProjects = ref([])

// Modal state
const projectModalVisible = ref(false)
const deleteModalVisible = ref(false)
const logModalVisible = ref(false)
const settingsModalVisible = ref(false)
const isEditing = ref(false)
const editingId = ref(null)
const deletingId = ref(null)
const logViewId = ref(null)
const logContentRef = ref(null)
let logTimer = null

// Settings
const autoStartEnabled = ref(false)

// View mode
const STORAGE_KEY = 'startup-manager-view-mode'
const viewMode = ref(localStorage.getItem(STORAGE_KEY) || 'card') // 'card' | 'table'

const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'card' ? 'table' : 'card'
  localStorage.setItem(STORAGE_KEY, viewMode.value)
}

// Form
const form = ref({
  name: '',
  directory: '',
  command: '',
  port: '',
  url: '',
  description: '',
  color: COLORS[0],
})

// Computed stats
const stats = computed(() => ({
  total: projects.value.length,
  running: projects.value.filter(p => statuses.value[p.id]?.running).length,
  stopped: projects.value.length - projects.value.filter(p => statuses.value[p.id]?.running).length,
}))

// Helpers
const isRunning = (id) => statuses.value[id]?.running || false
const getPid = (id) => statuses.value[id]?.pid
const getProjectName = (id) => {
  const p = projects.value.find(x => x.id === id)
  return p ? p.name : ''
}

const filterProjects = () => {
  const query = searchQuery.value.toLowerCase().trim()
  if (!query) {
    filteredProjects.value = projects.value
    return
  }
  filteredProjects.value = projects.value.filter(p =>
    p.name.toLowerCase().includes(query) ||
    p.command.toLowerCase().includes(query) ||
    p.directory.toLowerCase().includes(query) ||
    (p.description && p.description.toLowerCase().includes(query))
  )
}

// API calls
const loadProjects = async () => {
  try {
    projects.value = await invoke('get_projects')
    await pollStatuses()
    filterProjects()
  } catch (e) {
    showToast('加载失败: ' + e, 'error')
  }
}

const pollStatuses = async () => {
  try {
    const list = await invoke('get_all_statuses')
    statuses.value = {}
    list.forEach(s => statuses.value[s.id] = s)
  } catch (e) {
    // Ignore polling errors
  }
}

const startProject = async (id) => {
  try {
    await invoke('start_project', { id })
    showToast('项目已启动', 'success')
    await pollStatuses()
  } catch (e) {
    showToast('启动失败: ' + e, 'error')
  }
}

const stopProject = async (id) => {
  try {
    await invoke('stop_project', { id })
    showToast('项目已停止', 'info')
    await pollStatuses()
  } catch (e) {
    showToast('停止失败: ' + e, 'error')
  }
}

const saveProject = async () => {
  const { name, directory, command, port, url, description, color } = form.value
  if (!name.trim() || !directory.trim() || !command.trim()) {
    showToast('请填写项目名称、工作目录和启动命令', 'error')
    return
  }
  try {
    if (editingId.value) {
      projects.value = await invoke('update_project', {
        id: editingId.value,
        name,
        directory,
        command,
        description,
        color,
        port,
        url,
      })
      showToast('项目已更新', 'success')
    } else {
      projects.value = await invoke('add_project', {
        name,
        directory,
        command,
        description,
        color,
        port,
        url,
      })
      showToast('项目已创建', 'success')
    }
    closeModal()
    filterProjects()
  } catch (e) {
    showToast('保存失败: ' + e, 'error')
  }
}

const confirmDelete = async () => {
  if (!deletingId.value) return
  try {
    projects.value = await invoke('delete_project', { id: deletingId.value })
    showToast('项目已删除', 'info')
    closeDeleteModal()
    filterProjects()
  } catch (e) {
    showToast('删除失败: ' + e, 'error')
  }
}

const fetchLogs = async () => {
  if (!logViewId.value) return
  try {
    const logs = await invoke('get_project_logs', { id: logViewId.value })
    currentLogs.value = logs || ''
    // Scroll to bottom
    if (logContentRef.value) {
      nextTick(() => {
        logContentRef.value.scrollTop = logContentRef.value.scrollHeight
      })
    }
  } catch (e) {
    // Ignore
  }
}

const clearLogs = () => {
  currentLogs.value = ''
}

const openDirectory = async (id) => {
  const p = projects.value.find(x => x.id === id)
  if (p) {
    invoke('open_directory', { path: p.directory })
  }
}

const openUrl = (url) => {
  open(url)
}

// Modal actions
const openAddModal = () => {
  isEditing.value = false
  editingId.value = null
  form.value = {
    name: '',
    directory: '',
    command: '',
    port: '',
    url: '',
    description: '',
    color: COLORS[0],
  }
  projectModalVisible.value = true
}

const openEditModal = (id) => {
  const p = projects.value.find(x => x.id === id)
  if (!p) return
  isEditing.value = true
  editingId.value = id
  form.value = {
    name: p.name,
    directory: p.directory,
    command: p.command,
    port: p.port || '',
    url: p.url || '',
    description: p.description,
    color: p.color || COLORS[0],
  }
  projectModalVisible.value = true
}

const closeModal = () => {
  projectModalVisible.value = false
  editingId.value = null
}

const openDeleteModal = (id) => {
  deletingId.value = id
  deleteModalVisible.value = true
}

const closeDeleteModal = () => {
  deleteModalVisible.value = false
  deletingId.value = null
}

const currentLogs = ref('')
const logModalTitle = computed(() => getProjectName(logViewId.value))

const openLogModal = async (id) => {
  logViewId.value = id
  logModalVisible.value = true
  await fetchLogs()
  logTimer = setInterval(fetchLogs, 2000)
}

const closeLogModal = () => {
  logModalVisible.value = false
  logViewId.value = null
  currentLogs.value = ''
  if (logTimer) {
    clearInterval(logTimer)
    logTimer = null
  }
}

// Settings modal
const openSettingsModal = async () => {
  try {
    autoStartEnabled.value = await invoke('get_auto_start')
  } catch (e) {
    // Ignore
  }
  settingsModalVisible.value = true
}

const closeSettingsModal = () => {
  settingsModalVisible.value = false
}

const saveAutoStart = async () => {
  try {
    await invoke('set_auto_start', { enable: autoStartEnabled.value })
    showToast(
      autoStartEnabled.value ? '已开启开机自动启动' : '已关闭开机自动启动',
      'success'
    )
  } catch (e) {
    showToast('保存设置失败: ' + e, 'error')
  }
}

const toastIcon = (type) => {
  const icons = { success: '✓', error: '✗', info: 'ℹ' }
  return icons[type] || 'ℹ'
}

const removeToast = (id) => {
  const idx = toasts.findIndex(t => t.id === id)
  if (idx !== -1) {
    toasts.splice(idx, 1)
  }
}

// Lifecycle
onMounted(async () => {
  await loadProjects()
  filterProjects()
  setInterval(pollStatuses, 3000)
})

// Watch for changes
watch(projects, () => {
  filterProjects()
})

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    if (projectModalVisible.value) closeModal()
    if (deleteModalVisible.value) closeDeleteModal()
    if (logModalVisible.value) closeLogModal()
    if (settingsModalVisible.value) closeSettingsModal()
  }
  if (e.key === 'Enter' && projectModalVisible.value) {
    saveProject()
  }
})
</script>

<style>
/* Base styles are in style.css */
.header {
  padding: 20px 28px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary);
  -webkit-user-select: none;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
}

.logo {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, var(--accent), #7c3aed);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 700;
  color: #fff;
  box-shadow: 0 2px 12px rgba(74, 108, 247, 0.3);
}

.header h1 {
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.3px;
}

.header h1 span {
  color: var(--text-muted);
  font-weight: 400;
  font-size: 13px;
  margin-left: 10px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-card);
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn:hover {
  background: var(--bg-card-hover);
  border-color: var(--text-muted);
}

.btn-accent {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.btn-accent:hover {
  background: #3b5ce0;
  border-color: #3b5ce0;
  box-shadow: 0 2px 16px rgba(74, 108, 247, 0.3);
}

.btn-success {
  background: var(--success);
  border-color: var(--success);
  color: #fff;
}

.btn-success:hover {
  background: #16a34a;
  box-shadow: 0 2px 16px var(--success-glow);
}

.btn-danger {
  background: transparent;
  border-color: var(--danger);
  color: var(--danger);
}

.btn-danger:hover {
  background: var(--danger);
  color: #fff;
}

.btn-sm {
  padding: 5px 10px;
  font-size: 12px;
}

.btn-icon {
  padding: 6px 8px;
  font-size: 16px;
  line-height: 1;
}

.main {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}

.stats-bar {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 13px;
}

.stat-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.stat-dot.total {
  background: var(--accent);
}

.stat-dot.running {
  background: var(--success);
  animation: pulse 2s infinite;
}

.stat-dot.stopped {
  background: var(--text-muted);
}

@keyframes pulse {
  0%, 100% { opacity: 1; box-shadow: 0 0 0 0 var(--success-glow); }
  50% { opacity: 0.8; box-shadow: 0 0 0 6px transparent; }
}

.stat-value {
  font-weight: 700;
  font-family: 'JetBrains Mono', monospace;
}

.stat-label {
  color: var(--text-secondary);
}

.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(440px, 1fr));
  gap: 16px;
}

.project-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px;
  transition: all 0.25s;
  position: relative;
  overflow: hidden;
}

.project-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: var(--card-color, var(--accent));
  opacity: 0.6;
  transition: opacity 0.25s;
}

.project-card:hover {
  border-color: var(--text-muted);
  background: var(--bg-card-hover);
}

.project-card:hover::before {
  opacity: 1;
}

.project-card.running {
  border-color: rgba(34, 197, 94, 0.3);
  box-shadow: 0 0 24px var(--success-glow);
}

.project-card.running::before {
  background: var(--success);
  opacity: 1;
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
}

.card-title-group {
  flex: 1;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-badge.running {
  background: var(--success-glow);
  color: var(--success);
}

.status-badge.stopped {
  background: rgba(85, 85, 106, 0.15);
  color: var(--text-muted);
}

.card-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.card-actions {
  display: flex;
  gap: 4px;
}

.card-info {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-secondary);
  background: var(--bg-input);
  padding: 6px 10px;
  border-radius: 6px;
  word-break: break-all;
}

.info-icon {
  flex-shrink: 0;
  width: 16px;
  text-align: center;
  opacity: 0.6;
}

.card-footer {
  margin-top: 16px;
  display: flex;
  gap: 8px;
}

.card-footer .btn {
  flex: 1;
  justify-content: center;
}

.empty-state {
  text-align: center;
  padding: 80px 20px;
  color: var(--text-muted);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.3;
}

.empty-state h3 {
  font-size: 18px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.empty-state p {
  font-size: 14px;
  margin-bottom: 24px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 1000;
  align-items: center;
  justify-content: center;
  display: flex;
}

.modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  width: 520px;
  max-width: 90vw;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: var(--shadow);
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-group input,
.form-group textarea {
  padding: 10px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  font-family: 'JetBrains Mono', monospace;
  transition: border-color 0.2s;
  outline: none;
}

.form-group input:focus,
.form-group textarea:focus {
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.form-group textarea {
  min-height: 60px;
  resize: vertical;
  font-family: inherit;
}

.form-group .hint {
  font-size: 11px;
  color: var(--text-muted);
}

.color-picker {
  display: flex;
  gap: 8px;
}

.color-option {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.color-option:hover {
  transform: scale(1.15);
}

.color-option.active {
  border-color: #fff;
  box-shadow: 0 0 0 2px var(--bg-primary), 0 0 8px rgba(255, 255, 255, 0.2);
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.confirm-msg {
  padding: 24px;
  text-align: center;
}

.confirm-msg p {
  font-size: 14px;
  color: var(--text-secondary);
  margin-top: 8px;
}

.confirm-msg .project-name {
  color: var(--text-primary);
  font-weight: 600;
}

.toast-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 2000;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast {
  padding: 10px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 13px;
  box-shadow: var(--shadow);
  animation: toastIn 0.3s ease-out;
  display: flex;
  align-items: center;
  gap: 8px;
}

.toast.success {
  border-left: 3px solid var(--success);
}

.toast.error {
  border-left: 3px solid var(--danger);
}

.toast.info {
  border-left: 3px solid var(--accent);
}

@keyframes toastIn {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}

.toast-leave-active {
  animation: toastOut 0.3s forwards;
}

@keyframes toastOut {
  from { transform: translateX(0); opacity: 1; }
  to { transform: translateX(100%); opacity: 0; }
}

.search-box {
  position: relative;
}

.search-box input {
  padding: 8px 14px 8px 34px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
  width: 220px;
  outline: none;
  transition: all 0.2s;
}

.search-box input:focus {
  border-color: var(--border-focus);
  width: 280px;
}

.search-box .search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  font-size: 14px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.log-modal {
  width: 700px;
  max-width: 90vw;
}

.log-content {
  background: #0d0d14;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 16px;
  margin: 0;
  height: 400px;
  overflow-y: auto;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #c8c8d8;
  white-space: pre-wrap;
  word-break: break-all;
}

.log-empty {
  color: var(--text-muted);
  font-style: italic;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.setting-info {
  flex: 1;
}

.setting-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

/* Toggle switch */
.toggle {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--bg-input);
  border: 1px solid var(--border);
  transition: .2s;
  border-radius: 26px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: var(--text-muted);
  transition: .2s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--accent);
  border-color: var(--accent);
}

input:checked + .slider:before {
  transform: translateX(22px);
  background-color: #fff;
}

/* Table View */
.projects-table-container {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}

.projects-table {
  width: 100%;
  border-collapse: collapse;
}

.projects-table thead {
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.projects-table th {
  padding: 14px 16px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.projects-table tbody tr {
  border-bottom: 1px solid var(--border);
  transition: background 0.2s;
}

.projects-table tbody tr:last-child {
  border-bottom: none;
}

.projects-table tbody tr:hover {
  background: var(--bg-card-hover);
}

.projects-table tbody tr.running {
  background: linear-gradient(90deg, var(--success-glow), transparent);
}

.projects-table td {
  padding: 12px 16px;
  vertical-align: middle;
  white-space: nowrap;
}

.col-status {
  width: 110px;
  white-space: nowrap;
}

.col-name {
  min-width: 160px;
}

.col-command {
  min-width: 200px;
}

.col-port {
  width: 80px;
  white-space: nowrap;
}

.col-actions {
  width: 300px;
  white-space: nowrap;
}

.status-badge {
  white-space: nowrap;
}

.table-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.table-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
  white-space: normal;
}

.table-command {
  font-size: 13px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-primary);
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.table-directory {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  font-family: 'JetBrains Mono', monospace;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-actions .btn {
  margin: 0 2px;
  padding: 4px 8px;
}

.table-empty {
  padding: 40px 0 !important;
}
</style>
