# Single Stage: Build JAR and create custom Java runtime
FROM eclipse-temurin:21-alpine AS build

WORKDIR /app

# Copy the Gradle wrapper and build files
COPY gradle/wrapper/ /app/gradle/wrapper/
COPY gradlew gradlew.bat /app/
COPY build.gradle.kts settings.gradle.kts /app/

# Copy the source code
COPY src /app/src

# Build the JAR using Gradle
RUN ./gradlew build

# Analyze the JAR dependencies using jdeps, then create a custom runtime using jlink
RUN jdeps --ignore-missing-deps \
    -q  \
    --recursive  \
    --multi-release 21  \
    --print-module-deps  \
    --class-path 'build/classes/kotlin/main/META-INF/*'  \
    build/libs/eel-0.0.1-SNAPSHOT-standalone.jar > deps.txt
RUN $JAVA_HOME/bin/jlink \
        --add-modules $(cat deps.txt)\
        --add-modules jdk.crypto.cryptoki\
        --strip-debug \
        --no-man-pages \
        --no-header-files \
        --compress=zip-6 \
        --output /app/jre

# Final Stage: Minimal runtime to run the application
FROM alpine

# Copy the custom runtime and the JAR
COPY --from=build /app/jre /app/jre/
COPY --from=build /app/build/libs/eel-0.0.1-SNAPSHOT-standalone.jar /app/eel-bot.jar
# RUN mkdir -p /usr/local/newrelic
# ADD ./newrelic/newrelic.jar /usr/local/newrelic/newrelic.jar
# ADD ./newrelic/newrelic.yml /usr/local/newrelic/newrelic.yml

# Set the entrypoint for the application
# ENTRYPOINT ["/app/jre/bin/java", "-javaagent:/usr/local/newrelic/newrelic.jar", "-jar", "/app/eel-bot.jar"]
ENTRYPOINT ["/app/jre/bin/java", "-jar", "/app/eel-bot.jar"]
