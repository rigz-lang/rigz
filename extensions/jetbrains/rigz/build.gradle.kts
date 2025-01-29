plugins {
    id("org.jetbrains.intellij.platform") version "2.2.1"
}

repositories {
    mavenCentral()

    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaUltimate("2024.3.2.1")
    }
}