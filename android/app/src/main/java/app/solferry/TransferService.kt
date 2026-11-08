package app.solferry

import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.IBinder
import androidx.core.app.ServiceCompat

/** Keeps the process alive where a user-initiated job can't: older Android, or from the background. */
class TransferService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val count = intent?.getIntExtra(RUNNING, 1) ?: 1
        val percent = intent?.getIntExtra(PERCENT, 0) ?: 0
        val notification = Notifier.progress(this, count, percent)
        ServiceCompat.startForeground(this, ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        return START_NOT_STICKY
    }

    companion object {
        private const val ID = 1
        const val RUNNING = "running"
        const val PERCENT = "percent"
    }
}
