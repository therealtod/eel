plugins {
    kotlin("jvm") version "2.0.20"
}

group = "eelst.ilike"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-core:${Versions.JACKSON}")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-yaml:${Versions.JACKSON}")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:${Versions.JACKSON}")
    testImplementation(kotlin("test"))
    testImplementation("io.mockk:mockk:${Versions.MOCKK}")
}

tasks.test {
    useJUnitPlatform()
}
kotlin {
    jvmToolchain(21)
}

object Versions {
    const val JACKSON = "2.18.1"
    const val MOCKK = "1.13.13"
}