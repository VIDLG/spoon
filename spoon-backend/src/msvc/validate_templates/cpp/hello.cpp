#include <cstdio>
#include <string>
#include <vector>
#include <Windows.h>

int main() {
    std::vector<int> values{1, 2, 3};
    std::string msg = "spoon msvc validate";
    int width = GetSystemMetrics(SM_CXSCREEN);
    int height = GetSystemMetrics(SM_CYSCREEN);
    HWND desktop = GetDesktopWindow();
    std::printf("sample=%s\n", msg.c_str());
    std::printf("cpp_runtime=std::string+std::vector ok | values=%zu\n", values.size());
    std::printf("win32_api=GetSystemMetrics/GetDesktopWindow ok | screen=%dx%d | desktop=%s\n",
        width,
        height,
        desktop != nullptr ? "present" : "missing");
    std::printf("link_check=user32.lib ok\n");
    return width > 0 && height > 0 && !msg.empty() && values.size() == 3 ? 0 : 1;
}
