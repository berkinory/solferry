package app.solferry

import android.app.Application

class SolferryApp : Application() {
    lateinit var engine: Engine
        private set

    override fun onCreate() {
        super.onCreate()
        Notifier.createChannels(this)
        engine = Engine(this)
    }
}
