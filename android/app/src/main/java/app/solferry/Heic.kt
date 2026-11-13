package app.solferry

import android.content.Context
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.net.Uri
import android.util.Log
import androidx.exifinterface.media.ExifInterface
import java.io.File
import uniffi.solferry_ffi.DevicePlatform

/** Turns HEIC photos into JPEG for devices that can't open HEIC: Linux, and Windows without its Store extension. */
object Heic {
    private const val TAG = "SolferryHeic"
    private const val QUALITY = 95
    private val extensions = setOf("heic", "heif")
    private val mimeTypes = setOf("image/heic", "image/heif", "image/heic-sequence", "image/heif-sequence")

    fun wanted(platform: DevicePlatform?) = platform == DevicePlatform.LINUX || platform == DevicePlatform.WINDOWS

    fun matches(name: String, mime: String?) =
        name.substringAfterLast('.', "").lowercase() in extensions || mime?.lowercase() in mimeTypes

    /** The same basename with a .jpg extension. */
    fun jpegName(name: String): String {
        val stem = name.substringBeforeLast('.', name).takeIf { name.substringAfterLast('.', "").lowercase() in extensions }
        return "${stem ?: name}.jpg"
    }

    /** Writes the JPEG into [folder], or returns null so the original goes instead. */
    fun convert(context: Context, uri: Uri, name: String, folder: File): File? {
        val out = File(folder, jpegName(name))
        return try {
            // The decoder applies the container's rotation, so the pixels come out upright.
            val bitmap = ImageDecoder.decodeBitmap(ImageDecoder.createSource(context.contentResolver, uri)) { decoder, _, _ ->
                decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
            }
            try {
                out.outputStream().use { check(bitmap.compress(Bitmap.CompressFormat.JPEG, QUALITY, it)) }
            } finally {
                bitmap.recycle()
            }
            copyExif(context, uri, out)
            out
        } catch (error: Exception) {
            Log.w(TAG, "sending $name as HEIC, conversion failed", error)
            out.delete()
            null
        } catch (error: OutOfMemoryError) {
            Log.w(TAG, "sending $name as HEIC, too large to convert", error)
            out.delete()
            null
        }
    }

    private fun copyExif(context: Context, uri: Uri, out: File) {
        val source = context.contentResolver.openFileDescriptor(uri, "r")?.use { ExifInterface(it.fileDescriptor) } ?: return
        val target = ExifInterface(out)
        var copied = false
        for (tag in tags) {
            source.getAttribute(tag)?.let { target.setAttribute(tag, it); copied = true }
        }
        // The pixels are upright now, so the orientation that described the original would turn them twice.
        if (source.getAttribute(ExifInterface.TAG_ORIENTATION) != null) {
            target.setAttribute(ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL.toString())
            copied = true
        }
        if (copied) target.saveAttributes()
    }

    private val tags = listOf(
        ExifInterface.TAG_DATETIME, ExifInterface.TAG_DATETIME_ORIGINAL, ExifInterface.TAG_DATETIME_DIGITIZED,
        ExifInterface.TAG_OFFSET_TIME, ExifInterface.TAG_OFFSET_TIME_ORIGINAL, ExifInterface.TAG_OFFSET_TIME_DIGITIZED,
        ExifInterface.TAG_SUBSEC_TIME, ExifInterface.TAG_SUBSEC_TIME_ORIGINAL, ExifInterface.TAG_SUBSEC_TIME_DIGITIZED,
        ExifInterface.TAG_GPS_VERSION_ID, ExifInterface.TAG_GPS_LATITUDE_REF, ExifInterface.TAG_GPS_LATITUDE,
        ExifInterface.TAG_GPS_LONGITUDE_REF, ExifInterface.TAG_GPS_LONGITUDE, ExifInterface.TAG_GPS_ALTITUDE_REF,
        ExifInterface.TAG_GPS_ALTITUDE, ExifInterface.TAG_GPS_TIMESTAMP, ExifInterface.TAG_GPS_DATESTAMP,
        ExifInterface.TAG_GPS_SPEED_REF, ExifInterface.TAG_GPS_SPEED, ExifInterface.TAG_GPS_TRACK_REF, ExifInterface.TAG_GPS_TRACK,
        ExifInterface.TAG_GPS_IMG_DIRECTION_REF, ExifInterface.TAG_GPS_IMG_DIRECTION,
        ExifInterface.TAG_GPS_DEST_BEARING_REF, ExifInterface.TAG_GPS_DEST_BEARING,
        ExifInterface.TAG_GPS_H_POSITIONING_ERROR, ExifInterface.TAG_GPS_PROCESSING_METHOD, ExifInterface.TAG_GPS_MAP_DATUM,
        ExifInterface.TAG_GPS_MEASURE_MODE, ExifInterface.TAG_GPS_DOP, ExifInterface.TAG_GPS_SATELLITES, ExifInterface.TAG_GPS_STATUS,
        ExifInterface.TAG_GPS_DIFFERENTIAL, ExifInterface.TAG_GPS_AREA_INFORMATION,
        ExifInterface.TAG_MAKE, ExifInterface.TAG_MODEL, ExifInterface.TAG_SOFTWARE, ExifInterface.TAG_ARTIST,
        ExifInterface.TAG_COPYRIGHT, ExifInterface.TAG_IMAGE_DESCRIPTION, ExifInterface.TAG_USER_COMMENT,
        ExifInterface.TAG_IMAGE_UNIQUE_ID, ExifInterface.TAG_CAMERA_OWNER_NAME, ExifInterface.TAG_BODY_SERIAL_NUMBER,
        ExifInterface.TAG_LENS_MAKE, ExifInterface.TAG_LENS_MODEL, ExifInterface.TAG_LENS_SPECIFICATION,
        ExifInterface.TAG_LENS_SERIAL_NUMBER, ExifInterface.TAG_EXIF_VERSION,
        ExifInterface.TAG_EXPOSURE_TIME, ExifInterface.TAG_F_NUMBER, ExifInterface.TAG_APERTURE_VALUE,
        ExifInterface.TAG_MAX_APERTURE_VALUE, ExifInterface.TAG_SHUTTER_SPEED_VALUE, ExifInterface.TAG_BRIGHTNESS_VALUE,
        ExifInterface.TAG_EXPOSURE_BIAS_VALUE, ExifInterface.TAG_EXPOSURE_PROGRAM, ExifInterface.TAG_EXPOSURE_MODE,
        ExifInterface.TAG_PHOTOGRAPHIC_SENSITIVITY, ExifInterface.TAG_SENSITIVITY_TYPE, ExifInterface.TAG_ISO_SPEED,
        ExifInterface.TAG_FOCAL_LENGTH, ExifInterface.TAG_FOCAL_LENGTH_IN_35MM_FILM, ExifInterface.TAG_FLASH,
        ExifInterface.TAG_METERING_MODE, ExifInterface.TAG_LIGHT_SOURCE, ExifInterface.TAG_WHITE_BALANCE,
        ExifInterface.TAG_SCENE_CAPTURE_TYPE, ExifInterface.TAG_SCENE_TYPE, ExifInterface.TAG_SENSING_METHOD,
        ExifInterface.TAG_DIGITAL_ZOOM_RATIO, ExifInterface.TAG_CONTRAST, ExifInterface.TAG_SATURATION,
        ExifInterface.TAG_SHARPNESS, ExifInterface.TAG_SUBJECT_AREA, ExifInterface.TAG_SUBJECT_DISTANCE,
        ExifInterface.TAG_SUBJECT_DISTANCE_RANGE, ExifInterface.TAG_COLOR_SPACE,
    )
}
