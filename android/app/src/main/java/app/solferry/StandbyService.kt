package app.solferry

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.IBinder
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat

/** Optional. Android cuts a closed app off the network, so staying reachable takes a permanent notification. */
class StandbyService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val started = runCatching {
            ServiceCompat.startForeground(this, ID, Notifier.standby(this), ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE)
        }
        if (started.isFailure) stopSelf()
        return START_STICKY
    }

    companion object {
        const val ID = 3

        fun sync(context: Context, on: Boolean) {
            val intent = Intent(context, StandbyService::class.java)
            if (on) runCatching { ContextCompat.startForegroundService(context, intent) } else context.stopService(intent)
        }
    }
}
