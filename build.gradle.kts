plugins {
    kotlin("jvm") version "2.0.20"
    // id("org.beryx.jlink") version "3.1.1"
    application
}

application {
    mainClass.set("eelst.ilike.MainKt")
}

tasks {
    val fatJar = register<Jar>("fatJar") {
        dependsOn.addAll(
            listOf(
                "compileJava",
                "compileKotlin",
                "processResources"
            )
        ) // We need this for Gradle optimization to work
        archiveClassifier.set("standalone") // Naming the jar
        duplicatesStrategy = DuplicatesStrategy.EXCLUDE
        manifest { attributes(mapOf("Main-Class" to application.mainClass)) } // Provided we set it up in the application plugin configuration
        val sourcesMain = sourceSets.main.get()
        val contents = configurations.runtimeClasspath.get()
            .map { if (it.isDirectory) it else zipTree(it) } +
                sourcesMain.output
        from(contents)
    }
    build {
        dependsOn(fatJar) // Trigger fat jar creation during build
    }
}

group = "eelst.ilike"
version = "0.0.1-SNAPSHOT"

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
    // runtimeOnly("com.fasterxml.jackson.module:jackson-modules-java8:${Versions.JACKSON}")
    implementation("io.ktor:ktor-client-core:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-cio:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-resources:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-logging:${Versions.KTOR}")
    implementation("io.ktor:ktor-client-content-negotiation:${Versions.KTOR}")
    implementation("io.ktor:ktor-serialization-jackson:${Versions.KTOR}")
    testImplementation(kotlin("test"))
    testImplementation("io.mockk:mockk:${Versions.MOCKK}")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:${Versions.KOTLINX}")
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
    const val LOG4J = "2.23.1"
    const val KOTLINX = "1.10.1"
}