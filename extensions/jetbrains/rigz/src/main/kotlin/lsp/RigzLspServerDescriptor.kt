import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor


fun VirtualFile.isRigz(): Boolean {
  this.extension == "rg" || this.extension == "rigz"
}


internal class RigzLspServerSupportProvider : LspServerSupportProvider {
  override fun fileOpened(project: Project, file: VirtualFile, serverStarter: LspServerStarter) {
    if (file.isRigz()) {
      serverStarter.ensureServerStarted(RigzLspServerDescriptor(project))
    }
  }
}

private class RigzLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, "Rigz") {
  override fun isSupportedFile(file: VirtualFile) = file.isRigz()
  override fun createCommandLine() = GeneralCommandLine("rigz_lsp", "--stdio")
}