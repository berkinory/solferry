package app.solferry

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Accept and Decline on an offer's notification, answered without opening the app. */
class OfferAnswers : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val key = intent.getLongExtra(KEY, -1).takeIf { it >= 0 }?.toULong() ?: return
        (context.applicationContext as SolferryApp).engine.answerOffer(key, intent.getBooleanExtra(ACCEPT, false))
    }

    companion object {
        private const val KEY = "key"
        private const val ACCEPT = "accept"

        fun intent(context: Context, key: ULong, accept: Boolean): PendingIntent = PendingIntent.getBroadcast(
            context,
            key.toInt() * 2 + if (accept) 1 else 0,
            Intent(context, OfferAnswers::class.java).putExtra(KEY, key.toLong()).putExtra(ACCEPT, accept),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
    }
}
