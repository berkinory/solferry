package app.solferry

import android.Manifest
import android.app.Notification
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
    const val STANDBY = "standby"
    private const val OFFER = "offer"
    private const val PAIRING = "pairing"
    private var next = 100

    /** While someone is looking at the app, it shows these itself. */
    var appVisible = false

    fun createChannels(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(NotificationChannel(ASKS, "Requests and arrivals", NotificationManager.IMPORTANCE_HIGH))
        manager.createNotificationChannel(NotificationChannel(TRANSFERS, "Transfers in progress", NotificationManager.IMPORTANCE_LOW))
        manager.createNotificationChannel(NotificationChannel(STANDBY, "Receiving in the background", NotificationManager.IMPORTANCE_MIN))
    }

    fun opener(context: Context): PendingIntent = PendingIntent.getActivity(
        context, 0,
        Intent(context, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_IMMUTABLE,
    )

    fun progress(context: Context, count: Int, percent: Int): Notification =
        NotificationCompat.Builder(context, TRANSFERS)
            .setSmallIcon(R.drawable.ic_launcher_mark)
            .setContentTitle(if (count == 1) "Moving 1 transfer" else "Moving $count transfers")
            .setProgress(100, percent, false)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setContentIntent(opener(context))
            .build()

    fun standby(context: Context): Notification =
        NotificationCompat.Builder(context, STANDBY)
            .setSmallIcon(R.drawable.ic_launcher_mark)
            .setContentTitle("Ready to receive")
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setShowWhen(false)
            .setContentIntent(opener(context))
            .build()

    fun offer(context: Context, key: ULong, name: String) =
        show(context, "Incoming files", "$name wants to send you files", OFFER, key.toInt())

    fun pairing(context: Context, key: ULong, name: String) =
        show(context, "Pairing request", "$name wants to pair with this device", PAIRING, key.toInt())

    /** Takes back a question once it has been answered or the other device has left. */
    fun settleOffer(context: Context, key: ULong) = NotificationManagerCompat.from(context).cancel(OFFER, key.toInt())

    fun settlePairing(context: Context, key: ULong) = NotificationManagerCompat.from(context).cancel(PAIRING, key.toInt())

    fun text(context: Context, name: String, copied: Boolean) =
        show(context, "Text from $name", if (copied) "Copied to your clipboard" else "Open Solferry to copy it")

    fun received(context: Context, label: String, name: String) =
        show(context, "Files received", "$label from $name, in Downloads/Solferry")

    private fun allowed(context: Context) =
        ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED

    /** Swaps what an ongoing notification says, for a service that is already running. */
    fun replace(context: Context, id: Int, notification: Notification) {
        if (allowed(context)) NotificationManagerCompat.from(context).notify(id, notification)
    }

    private fun show(context: Context, title: String, body: String, tag: String? = null, id: Int = next++) {
        if (appVisible || !allowed(context)) return
        val notification = NotificationCompat.Builder(context, ASKS)
            .setSmallIcon(R.drawable.ic_launcher_mark)
            .setContentTitle(title)
            .setContentText(body)
            .setContentIntent(opener(context))
            .setAutoCancel(true)
            .build()
        NotificationManagerCompat.from(context).notify(tag, id, notification)
    }
}
