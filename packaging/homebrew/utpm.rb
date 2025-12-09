class Utpm < Formula
  desc "Unofficial Typst Package Manager"
  homepage "https://github.com/typst-community/utpm"
  url "https://github.com/typst-community/utpm/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "SKIP"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
    
    # Generate and install shell completions
    generate_completions_from_executable(bin/"utpm", "generate")
    
    # Install documentation
    doc.install "docs/GUIDE.md"
  end

  test do
    assert_match "utpm", shell_output("#{bin}/utpm --version")
  end
end
