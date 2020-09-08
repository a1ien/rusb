package com.github.a1ien.rusb.example.rustandroidusb

import android.content.Context
import android.hardware.usb.UsbManager
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import kotlinx.android.synthetic.main.activity_main.*

class MainActivity : AppCompatActivity() {
    private val tag: String = this.javaClass.name

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val usbManager = getSystemService(Context.USB_SERVICE) as UsbManager
        val cons = usbManager!!.deviceList.values.map { usbManager!!.openDevice(it) }
        val fds = cons.map { it.fileDescriptor }

        // Invoke Rust FFI
        init()
        val androidDevs = listAndroid(fds.toIntArray())
        val nativeDevs = listNative()
        Log.i(tag, nativeDevs)
        Log.i(tag, androidDevs)

        setContentView(R.layout.activity_main)
        
        tbAndroid.text = androidDevs
        tbNative.text = nativeDevs
    }

    // https://medium.com/visly/rust-on-android-19f34a2fb43
    init {
        try {
            System.loadLibrary("androidlib")
            Log.i(tag, "Loaded native library!")
        } catch (ex: Exception) {
            Log.e(tag, "Failed to load native library!", ex)
        }
    }
    external fun listAndroid(fds: IntArray): String
    external fun listNative(): String
    external fun init()
}