class Utpm < Formula
  desc "Unofficial Typst Package Manager"
  homepage "https://github.com/typst-community/utpm"
  url "https://github.com/typst-community/utpm/archive/refs/tags/vv0.4.0.tar.gz"
  sha256 "60634e2276d0632a25decdd13e595c750710a82075c0dc19599d22aa688d08d9"
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
