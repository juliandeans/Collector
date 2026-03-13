export function createDebouncedJob(delay = 600) {
  let timeoutId = null;
  let pendingJob = null;

  return {
    schedule(job, onFire) {
      clearTimeout(timeoutId);
      pendingJob = job;

      timeoutId = setTimeout(() => {
        const nextJob = pendingJob;
        timeoutId = null;
        pendingJob = null;
        if (nextJob) {
          onFire?.(nextJob);
        }
      }, delay);
    },

    flush() {
      if (!timeoutId || !pendingJob) return null;

      clearTimeout(timeoutId);
      timeoutId = null;
      const job = pendingJob;
      pendingJob = null;
      return job;
    },

    clear() {
      clearTimeout(timeoutId);
      timeoutId = null;
      pendingJob = null;
    },
  };
}
