<script>
  import { onMount, onDestroy } from 'svelte'
  import { revealItemInDir } from '@tauri-apps/plugin-opener'

  let jobs = $state([])
  let interval

  async function fetchJobs() {
    try {
      const res = await fetch('http://localhost:14200/api/jobs')
      if (res.ok) {
        jobs = await res.json()
      }
    } catch (e) {
      console.error(e)
    }
  }

  async function openFolder(path) {
    try {
      await revealItemInDir(path)
    } catch (e) {
      console.error('Failed to open folder:', e)
    }
  }

  onMount(() => {
    fetchJobs()
    interval = setInterval(fetchJobs, 1000)
  })

  onDestroy(() => {
    clearInterval(interval)
  })
</script>

<div class="p-4 bg-white rounded-lg shadow-md">
  <h2 class="text-xl font-bold mb-4">Job History</h2>
  <div class="overflow-x-auto">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >ID</th
          >
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >URL</th
          >
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >Status</th
          >
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >Actions</th
          >
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#each jobs as job (job.id)}
          <tr>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500"
              >{job.id.slice(0, 8)}...</td
            >
            <td
              class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 truncate max-w-xs"
              >{job.url}</td
            >
            <td class="px-6 py-4 whitespace-nowrap">
              <span
                class={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full 
                ${
                  job.status === 'completed'
                    ? 'bg-green-100 text-green-800'
                    : job.status === 'processing'
                      ? 'bg-yellow-100 text-yellow-800'
                      : job.status === 'queued'
                        ? 'bg-gray-100 text-gray-800'
                        : 'bg-red-100 text-red-800'
                }`}
              >
                {job.status}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {#if job.status === 'completed' && job.result_path}
                <button
                  onclick={() => openFolder(job.result_path)}
                  class="text-blue-600 hover:text-blue-900"
                  title="Open Output Folder"
                >
                  <!-- Folder Icon -->
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke-width="1.5"
                    stroke="currentColor"
                    class="w-5 h-5"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z"
                    />
                  </svg>
                </button>
              {/if}
            </td>
          </tr>
        {/each}
        {#if jobs.length === 0}
          <tr>
            <td colspan="4" class="px-6 py-4 text-center text-gray-500"
              >No jobs found.</td
            >
          </tr>
        {/if}
      </tbody>
    </table>
  </div>
</div>
