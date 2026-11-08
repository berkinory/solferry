package app.solferry

import android.app.Notification
import android.app.job.JobInfo
import android.app.job.JobParameters
import android.app.job.JobScheduler
import android.app.job.JobService
import android.content.ComponentName
import android.content.Context
import android.net.NetworkRequest
import android.os.Build
import androidx.annotation.RequiresApi

/** One user-initiated job that stays open while anything moves, so Android doesn't cut it short. */
@RequiresApi(Build.VERSION_CODES.UPSIDE_DOWN_CAKE)
class TransferJob : JobService() {
    override fun onStartJob(params: JobParameters): Boolean {
        val notification = latest ?: return false
        running = this to params
        setNotification(params, NOTIFICATION, notification, JOB_END_NOTIFICATION_POLICY_REMOVE)
        return true
    }

    override fun onStopJob(params: JobParameters): Boolean {
        running = null
        scheduled = false
        if (params.stopReason == JobParameters.STOP_REASON_USER) (application as SolferryApp).engine.cancelRunning()
        return false
    }

    companion object {
        private const val JOB = 1
        private const val NOTIFICATION = 2
        private var running: Pair<TransferJob, JobParameters>? = null
        private var scheduled = false
        private var latest: Notification? = null

        /** False when Android refuses the job, which it does unless the app is on screen. */
        fun keep(context: Context, notification: Notification, bytes: Long): Boolean {
            latest = notification
            running?.let { (job, params) ->
                job.setNotification(params, NOTIFICATION, notification, JOB_END_NOTIFICATION_POLICY_REMOVE)
                return true
            }
            if (scheduled) return true
            val job = JobInfo.Builder(JOB, ComponentName(context, TransferJob::class.java))
                .setUserInitiated(true)
                .setRequiredNetwork(NetworkRequest.Builder().clearCapabilities().build())
                .setEstimatedNetworkBytes(bytes, bytes)
                .build()
            val scheduler = context.getSystemService(JobScheduler::class.java)
            scheduled = runCatching { scheduler.schedule(job) == JobScheduler.RESULT_SUCCESS }.getOrDefault(false)
            return scheduled
        }

        fun finish(context: Context) {
            val open = running
            if (open != null) open.first.jobFinished(open.second, false)
            else if (scheduled) context.getSystemService(JobScheduler::class.java).cancel(JOB)
            running = null
            scheduled = false
            latest = null
        }
    }
}
