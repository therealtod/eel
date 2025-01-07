FROM openjdk:21-jdk-slim AS build
WORKDIR /usr/app/
COPY . .
RUN ./gradlew build

# Package stage

FROM eclipse-temurin:21-jre
WORKDIR $APP_HOME
COPY --from=build /usr/app/build/libs/$BUILD_JAR_NAME $OUTPUT_JAR_NAME
ENTRYPOINT exec java -jar /$BUILD_JAR_NAME
