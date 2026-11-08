package app.solferry

import android.content.Context
import android.content.Intent
import android.os.Build

/** Holds the process open while transfers run, with one quiet notification showing progress. */
object Keepalive {
    private var serviceUp = false
    private var shown: Pair<Int, Int>? = null

    fun sync(context: Context, transfers: List<Transfer>, standby: Boolean) {
        val running = transfers.filter { it.running }
        val newer = Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE
        if (running.isEmpty()) {
            if (shown == null) return
            if (newer) TransferJob.finish(context)
            if (serviceUp) context.stopService(Intent(context, TransferService::class.java))
            if (standby) Notifier.replace(context, StandbyService.ID, Notifier.standby(context))
            serviceUp = false
            shown = null
            return
        }
        val total = running.sumOf { it.total }.coerceAtLeast(1)
        val percent = (running.sumOf { it.transferred } * 100 / total).toInt()
        if (shown == running.size to percent) return
        shown = running.size to percent
        if (standby) return Notifier.replace(context, StandbyService.ID, Notifier.progress(context, running.size, percent))
        if (newer && !serviceUp && TransferJob.keep(context, Notifier.progress(context, running.size, percent), total)) return
        val intent = Intent(context, TransferService::class.java)
            .putExtra(TransferService.RUNNING, running.size)
            .putExtra(TransferService.PERCENT, percent)
        runCatching { context.startForegroundService(intent) }.onSuccess { serviceUp = true }
    }
}
