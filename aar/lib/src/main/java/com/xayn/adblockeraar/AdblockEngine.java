package com.xayn.adblockeraar;

import org.jetbrains.annotations.NotNull;

import java.io.IOException;
import java.io.InputStream;

public interface AdblockEngine {
    AdblockResult match(String url, String host, String type);

    void destroy();

    boolean deserialize(InputStream stream) throws IOException;

    boolean deserialize(String filePath);

    void enableTag(@NotNull String tag);

    void disableTag(@NotNull String tag);

    boolean hasTag(@NotNull String tag);
}

