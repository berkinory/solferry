package app.solferry

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat

/** Keeps the process alive while files move, with one quiet notification that shows progress. */
class TransferService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val count = intent?.getIntExtra(RUNNING, 1) ?: 1
        val percent = intent?.getIntExtra(PERCENT, 0) ?: 0
        val notification = NotificationCompat.Builder(this, Notifier.TRANSFERS)
            .setSmallIcon(R.drawable.ic_launcher_mark)
            .setContentTitle(if (count == 1) "Moving 1 transfer" else "Moving $count transfers")
            .setProgress(100, percent, false)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setContentIntent(Notifier.opener(this))
            .build()
        val type = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC else 0
        ServiceCompat.startForeground(this, ID, notification, type)
        return START_NOT_STICKY
    }

    companion object {
        private const val ID = 1
        private const val RUNNING = "running"
        private const val PERCENT = "percent"
        private var active = false
        private var shown = 0 to 0

        fun sync(context: Context, transfers: List<Transfer>) {
            val running = transfers.filter { it.running }
            if (running.isEmpty()) {
                if (active) context.stopService(Intent(context, TransferService::class.java))
                active = false
                return
            }
            val total = running.sumOf { it.total }.coerceAtLeast(1)
            val percent = (running.sumOf { it.transferred } * 100 / total).toInt()
            if (active && shown == running.size to percent) return
            shown = running.size to percent
            val intent = Intent(context, TransferService::class.java)
                .putExtra(RUNNING, running.size)
                .putExtra(PERCENT, percent)
            runCatching { context.startForegroundService(intent) }.onSuccess { active = true }
        }
    }
}
