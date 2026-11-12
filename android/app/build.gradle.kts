plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
}

val rustOut = layout.buildDirectory.dir("generated/rust")

val buildRust = tasks.register<Exec>("buildRust") {
    description = "Builds the Rust core for Android and generates its Kotlin bindings."
    workingDir = rootDir
    commandLine("./build-rust.sh", rustOut.get().asFile.absolutePath)
    inputs.dir(rootDir.resolve("../crates"))
    inputs.file(rootDir.resolve("../Cargo.lock"))
    inputs.property("abis", providers.environmentVariable("SOLFERRY_ABIS").orElse(""))
    inputs.property("profile", providers.environmentVariable("SOLFERRY_RUST_PROFILE").orElse(""))
    outputs.dir(rustOut)
}

val abis = providers.environmentVariable("SOLFERRY_ABIS").orElse("arm64-v8a,x86_64").get().split(",")

val releaseVersion = Regex("""(?m)^version = "([^"]+)"""")
    .find(rootDir.resolve("../Cargo.toml").readText())!!.groupValues[1]

android {
    namespace = "app.solferry"
    compileSdk = 37

    defaultConfig {
        applicationId = "app.solferry"
        minSdk = 29
        targetSdk = 36
        versionCode = releaseVersion.split(".").fold(0) { code, part -> code * 100 + part.toInt() }
        versionName = releaseVersion
        // Keeps only the ABIs the Rust core is built for, dropping JNA's copies for the rest.
        ndk { abiFilters += abis }
    }

    signingConfigs {
        create("release") {
            providers.environmentVariable("SOLFERRY_KEYSTORE").orNull?.let { path ->
                storeFile = file(path)
                storePassword = providers.environmentVariable("SOLFERRY_KEYSTORE_PASSWORD").get()
                keyAlias = "solferry"
                keyPassword = storePassword
            }
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs["release"].takeIf { it.storeFile != null }
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }

    buildFeatures {
        buildConfig = true
        compose = true
    }

    sourceSets["main"].apply {
        kotlin.directories += listOf(
            rustOut.get().dir("kotlin").asFile.path,
            rootDir.resolve("../design/generated").path,
        )
        jniLibs.directories += rustOut.get().dir("jniLibs").asFile.path
    }

    packaging {
        jniLibs.useLegacyPackaging = false
    }
}

tasks.named("preBuild") { dependsOn(buildRust) }

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.activity.compose)
    implementation(libs.lifecycle.viewmodel.compose)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.core.ktx)
    implementation("${libs.jna.get()}@aar")
}
