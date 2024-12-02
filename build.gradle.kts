plugins {
    kotlin("jvm") version "2.0.20"
}

group = "eelst.ilike"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.apache.logging.log4j:log4j-core:${Versions.LOG4J}")
    implementation("org.apache.logging.log4j:log4j-api-kotlin:1.5.0")
    runtimeOnly("org.apache.logging.log4j:log4j-slf4j2-impl:${Versions.LOG4J}")
    implementation("com.fasterxml.jackson.core:jackson-core:${Versions.JACKSON}")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-yaml:${Versions.JACKSON}")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:${Versions.JACKSON}")
    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.9.8")
    runtimeOnly("com.fasterxml.jackson.module:jackson-modules-java8:${Versions.JACKSON}")
    implementation("io.ktor:ktor-client-core:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-cio:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-resources:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-logging:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-content-negotiation:${Versions.KTOR}")
    implementation("io.ktor:ktor-serialization-jackson:${Versions.KTOR}")
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
    const val JACKSON = "2.18.2"
    const val MOCKK = "1.13.13"
    const val KTOR = "3.0.1"
    const val LOG4J = "3.0.0-beta2"
}