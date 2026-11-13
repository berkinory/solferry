package app.solferry

import android.graphics.drawable.Icon
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import kotlinx.coroutines.Job
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import uniffi.solferry_ffi.Reach

/** The same switch as the Visibility card in settings. Long-pressing it opens the app. */
class VisibilityTile : TileService() {
    private val scope = MainScope()
    private var watching: Job? = null

    private val engine get() = (application as SolferryApp).engine

    override fun onStartListening() {
        watching = scope.launch {
            engine.state.map { it.me?.visibility }.distinctUntilChanged().collect(::show)
        }
    }

    override fun onStopListening() {
        watching?.cancel()
        watching = null
        // Pulling down the shade starts the process and with it the node, which shouldn't linger unseen.
        if (!Notifier.appVisible) engine.leaveUnlessStaying()
    }

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    override fun onClick() {
        val visibility = engine.state.value.me?.visibility ?: return
        engine.setReach(if (visibility == HIDDEN) Reach.EVERYONE else Reach.HIDDEN)
        engine.resumeStandby()
    }

    /** Null while the node starts, which takes a moment after the shade wakes the process. */
    private fun show(visibility: String?) {
        val tile = qsTile ?: return
        tile.icon = Icon.createWithResource(this, R.drawable.ic_mark)
        tile.label = getString(R.string.tile_label)
        tile.state = when (visibility) {
            null -> Tile.STATE_UNAVAILABLE
            HIDDEN -> Tile.STATE_INACTIVE
            else -> Tile.STATE_ACTIVE
        }
        tile.subtitle = visibility?.let { getString(if (it == HIDDEN) R.string.tile_hidden else R.string.tile_visible) }
        tile.updateTile()
    }

    private companion object {
        /** How the core names [Reach.HIDDEN] in [AppState.me]. */
        const val HIDDEN = "Hidden"
    }
}
