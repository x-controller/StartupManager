import { reactive } from 'vue'

const toasts = reactive([])
let toastId = 0

function showToast(msg, type = 'info') {
  const id = ++toastId
  toasts.push({ id, msg, type, leaving: false })
  setTimeout(() => {
    const t = toasts.find(t => t.id === id)
    if (t) t.leaving = true
    setTimeout(() => {
      const idx = toasts.findIndex(t => t.id === id)
      if (idx !== -1) toasts.splice(idx, 1)
    }, 300)
  }, 3000)
}

export function useToast() {
  return { toasts, showToast }
}
