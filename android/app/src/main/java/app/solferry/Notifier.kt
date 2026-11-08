package app.solferry

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat

/** System notifications for things that need a person: offers, pairing, shared text and arrivals. */
object Notifier {
    const val ASKS = "asks"
    const val TRANSFERS = "transfers"
    private var next = 100

    fun createChannels(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(NotificationChannel(ASKS, "Requests and arrivals", NotificationManager.IMPORTANCE_HIGH))
        manager.createNotificationChannel(NotificationChannel(TRANSFERS, "Transfers in progress", NotificationManager.IMPORTANCE_LOW))
    }

    fun opener(context: Context): PendingIntent = PendingIntent.getActivity(
        context, 0,
        Intent(context, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE,
    )

    fun offer(context: Context, name: String) = show(context, "Incoming files", "$name wants to send you files")

    fun pairing(context: Context, name: String) = show(context, "Pairing request", "$name wants to pair with this device")

    fun text(context: Context, name: String, copied: Boolean) =
        show(context, "Text from $name", if (copied) "Copied to your clipboard" else "Open Solferry to copy it")

    fun received(context: Context, label: String, name: String) =
        show(context, "Files received", "$label from $name, in Downloads/Solferry")

    private fun show(context: Context, title: String, body: String) {
        val allowed = ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED
        if (!allowed) return
        val notification = NotificationCompat.Builder(context, ASKS)
            .setSmallIcon(R.drawable.ic_launcher_mark)
            .setContentTitle(title)
            .setContentText(body)
            .setContentIntent(opener(context))
            .setAutoCancel(true)
            .build()
        NotificationManagerCompat.from(context).notify(next++, notification)
    }
}
