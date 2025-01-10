/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package org.mozilla.appservices.tooling.nimbus

import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.file.Directory
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.MapProperty
import org.gradle.api.provider.Property
import org.gradle.api.provider.Provider

abstract class NimbusPluginExtension {
    /**
     * The .fml.yaml manifest file.
     *
     * If absent this defaults to `nimbus.fml.yaml`.
     * If relative, it is relative to the project root.
     *
     * @return
     */
    abstract Property<String> getManifestFile()

    /**
     * The mapping between the build variant and the release channel.
     *
     * Variants that are not in this map are used literally.
     * @return
     */
    abstract MapProperty<String, String> getChannels()

    /**
     * The filename of the manifest ingested by Experimenter.
     *
     * If this is a relative name, it is taken to be relative to the project's root directory.
     *
     * If missing, this defaults to `.experimenter.json`.
     * @return
     */
    abstract Property<String> getExperimenterManifest()

    /**
     * The directory to which the generated files should be written.
     *
     * This defaults to the generated sources folder in the build directory.
     *
     * @return
     */
    abstract Property<String> getOutputDir()

    /**
     * The file(s) containing the version(s)/ref(s)/location(s) for additional repositories.
     *
     * This defaults to an empty list.
     *
     * @return
     */
    abstract ListProperty<String> getRepoFiles()

    /**
     * The directory where downloaded files are or where they should be cached.
     *
     * If missing, this defaults to the Nimbus cache folder in the build directory.
     *
     * @return
     */
    abstract Property<String> getCacheDir()

    /**
     * The directory where a local installation of application services can be found.
     *
     * This defaults to `null`, in which case the plugin will download a copy of the correct
     * nimbus-fml binary for this version of the plugin.
     *
     * @return
     */
    abstract Property<String> getApplicationServicesDir()
}

class NimbusPlugin implements Plugin<Project> {

    void apply(Project project) {
        def extension = project.extensions.create('nimbus', NimbusPluginExtension)

        // Configure default values ("conventions") for our
        // extension properties.
        extension.manifestFile.convention('nimbus.fml.yaml')
        extension.cacheDir.convention('nimbus-cache')

        def assembleToolsTask = setupAssembleNimbusTools(project)

        def validateTask = setupValidateTask(project)
        validateTask.configure {
            // Gradle tracks the dependency on the `nimbus-fml` binary that the
            // `assembleNimbusTools` task produces implicitly; we don't need an
            // explicit `dependsOn` here.
            fmlBinary = assembleToolsTask.flatMap { it.fmlBinary }
        }

        if (project.hasProperty('android')) {
            // If the Android Gradle Plugin is configured, add the sources
            // generated by the `nimbusFeatures{variant}` task to the sources
            // for that variant. `variant.sources` is the modern, lazy
            // replacement for the deprecated `registerJavaGeneratingTask` API.
            def androidComponents = project.extensions.getByName('androidComponents')
            androidComponents.onVariants(androidComponents.selector().all()) { variant ->
                def generateTask = setupNimbusFeatureTasks(variant, project)

                generateTask.configure {
                    fmlBinary = assembleToolsTask.flatMap { it.fmlBinary }
                    dependsOn validateTask
                }

                variant.sources.java.addGeneratedSourceDirectory(generateTask) { it.outputDir }
            }
        } else {
            // Otherwise, if we aren't building for Android, add an explicit
            // dependency on the `nimbusFeatures` task to each `*compile*`
            // task.
            def generateTask = setupNimbusFeatureTasks([
                    name: project.name
            ], project)

            generateTask.configure {
                fmlBinary = assembleToolsTask.flatMap { it.fmlBinary }
                dependsOn validateTask
            }

            project.tasks.named {
                it.contains('compile')
            }.configureEach { task ->
                task.dependsOn generateTask
            }
        }
    }

    def setupAssembleNimbusTools(Project project) {
        def applicationServicesDir = project.nimbus.applicationServicesDir
        return project.tasks.register('assembleNimbusTools', NimbusAssembleToolsTask) { task ->
            group "Nimbus"
            description "Fetch the Nimbus FML tools from Application Services"

            def asVersion = getProjectVersion()
            def fmlRoot = getFMLRoot(project, asVersion)

            archiveFile = fmlRoot.map { it.file('nimbus-fml.zip') }
            hashFile = fmlRoot.map { it.file('nimbus-fml.sha256') }
            fmlBinary = fmlRoot.map { it.file(getFMLFile()) }

            fetch {
                // Try archive.mozilla.org release first
                archive = "https://archive.mozilla.org/pub/app-services/releases/$asVersion/nimbus-fml.zip"
                hash = "https://archive.mozilla.org/pub/app-services/releases/$asVersion/nimbus-fml.sha256"

                // Fall back to a nightly release
                fallback {
                    archive = "https://firefox-ci-tc.services.mozilla.com/api/index/v1/task/project.application-services.v2.nimbus-fml.$asVersion/artifacts/public/build/nimbus-fml.zip"
                    hash = "https://firefox-ci-tc.services.mozilla.com/api/index/v1/task/project.application-services.v2.nimbus-fml.$asVersion/artifacts/public/build/nimbus-fml.sha256"
                }
            }

            unzip {
                include "${getArchOs()}*/release/nimbus-fml*"
            }

            onlyIf('`applicationServicesDir` == null') {
                applicationServicesDir.getOrNull() == null
            }
        }
    }

    /**
     * The directory where nimbus-fml will live.
     * We put it in a build directory so we refresh it on a clean build.
     * @param project
     * @param version
     * @return
     */
    static Provider<Directory> getFMLRoot(Project project, String version) {
        return project.layout.buildDirectory.dir("bin/nimbus/$version")
    }

    static def getArchOs() {
        String osPart
        String os = System.getProperty("os.name").toLowerCase()
        if (os.contains("win")) {
            osPart = "pc-windows-gnu"
        } else if (os.contains("nix") || os.contains("nux") || os.contains("aix")) {
            osPart = "unknown-linux"
        } else if (os.contains("mac")) {
            osPart = "apple-darwin"
        } else {
            osPart = "unknown"
        }

        String arch = System.getProperty("os.arch").toLowerCase()
        String archPart
        if (arch.contains("x86_64")) {
            archPart = "x86_64"
        } else if (arch.contains("amd64")) {
            archPart = "x86_64"
        } else if (arch.contains("aarch")) {
            archPart = "aarch64"
        } else {
            archPart = "unknown"
        }
        println("OS and architecture detected as $os on $arch")
        return "${archPart}-${osPart}"
    }

    static String getFMLFile() {
        String os = System.getProperty("os.name").toLowerCase()
        String binaryName = "nimbus-fml"
        if (os.contains("win")) {
            binaryName = "nimbus-fml.exe"
        }
        return binaryName
    }

    String getProjectVersion() {
        Properties props = new Properties()
        def stream = getClass().getResourceAsStream("/nimbus-gradle-plugin.properties")
        stream.withStream { props.load(it) }
        return props.get("version")
    }

    def setupNimbusFeatureTasks(Object variant, Project project) {
        return project.tasks.register("nimbusFeatures${variant.name.capitalize()}", NimbusFeaturesTask) {
            description = "Generate Kotlin data classes for Nimbus enabled features"
            group = "Nimbus"

            doFirst {
                println("Nimbus FML generating Kotlin")
                println("manifest             ${inputFile.get().asFile}")
                println("cache dir            ${cacheDir.get().asFile}")
                println("repo file(s)         ${repoFiles.files.join()}")
                println("channel              ${channel.get()}")
            }

            doLast {
                println("outputFile    ${outputDir.get().asFile}")
            }

            projectDir = project.rootDir.toString()
            repoFiles = project.files(project.nimbus.repoFiles)
            applicationServicesDir = project.nimbus.applicationServicesDir
            inputFile = project.layout.projectDirectory.file(project.nimbus.manifestFile)
            cacheDir = project.layout.buildDirectory.dir(project.nimbus.cacheDir).map {
                // The `nimbusFeatures*` and `nimbusValidate` tasks can
                // technically use the same cache directory, but Gradle
                // discourages this, because such "overlapping outputs"
                // inhibit caching and parallelization
                // (https://github.com/gradle/gradle/issues/28394).
                it.dir("features${variant.name.capitalize()}")
            }
            channel = project.nimbus.channels.getting(variant.name).orElse(variant.name)
            outputDir = project.layout.buildDirectory.dir("generated/source/nimbus/${variant.name}/kotlin")
        }
    }

    def setupValidateTask(Project project) {
        return project.tasks.register('nimbusValidate', NimbusValidateTask) {
            description = "Validate the Nimbus feature manifest for the app"
            group = "Nimbus"

            doFirst {
                println("Nimbus FML: validating manifest")
                println("manifest             ${inputFile.get().asFile}")
                println("cache dir            ${cacheDir.get().asFile}")
                println("repo file(s)         ${repoFiles.files.join()}")
            }

            projectDir = project.rootDir.toString()
            repoFiles = project.files(project.nimbus.repoFiles)
            applicationServicesDir = project.nimbus.applicationServicesDir
            inputFile = project.layout.projectDirectory.file(project.nimbus.manifestFile)
            cacheDir = project.layout.buildDirectory.dir(project.nimbus.cacheDir).map {
                it.dir('validate')
            }

            // `nimbusValidate` doesn't have any outputs, so Gradle will always
            // run it, even if its inputs haven't changed. This predicate tells
            // Gradle to ignore the outputs, and only consider the inputs, for
            // up-to-date checks.
            outputs.upToDateWhen { true }
        }
    }

}
