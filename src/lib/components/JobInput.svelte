<script>
  let url = $state('')
  let loading = $state(false)

  async function handleSubmit() {
    if (!url) return
    loading = true
    try {
      const res = await fetch('http://localhost:14200/api/transcribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url }),
      })
      if (res.ok) {
        url = ''
      } else {
        console.error('Failed to submit job')
      }
    } catch (e) {
      console.error(e)
    } finally {
      loading = false
    }
  }
</script>

<div class="p-4 bg-white rounded-lg shadow-md mb-6">
  <h2 class="text-xl font-bold mb-4">New Transcription Job</h2>
  <div class="flex gap-2">
    <input
      type="text"
      bind:value={url}
      placeholder="Enter YouTube URL..."
      class="flex-1 p-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
    <button
      onclick={handleSubmit}
      disabled={loading}
      class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
    >
      {loading ? 'Submitting...' : 'Transcribe'}
    </button>
  </div>
</div>
