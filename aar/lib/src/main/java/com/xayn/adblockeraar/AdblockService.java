package com.xayn.adblockeraar;

 public interface AdblockService {
     AdblockEngine createEngine(String rules);

     AdblockEngine createEngine();
 }
