package przemyk.gansquest;

import java.util.ArrayList;
import org.libsdl.app.SDLActivity;
import android.content.Intent;
import android.net.Uri;
import androidx.documentfile.provider.DocumentFile;

public class MyActivity extends SDLActivity {
    public boolean pickDir = false;

    protected String[] getLibraries() {
        return new String[] { "SDL3", "gansquest" };
    }

    // Spaghetti based on https://github.com/libsdl-org/SDL/pull/9687
    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (((MyActivity) mSingleton).pickDir == true) {
            ((MyActivity) mSingleton).pickDir = false;
            Uri treeUri = data.getData();

            getContentResolver().takePersistableUriPermission(
                    treeUri,
                    (Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            );

            MyActivity.onNativePickDirectory(treeUri.toString());

            // int flags = data.getFlags();

            // int takeFlags =
            //         flags & (Intent.FLAG_GRANT_READ_URI_PERMISSION
            //                | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);

            // getContentResolver().takePersistableUriPermission(treeUri, takeFlags);

        }
    }

    public static native void onNativePickDirectory(String dir);

    // waiting for https://github.com/libsdl-org/SDL/issues/9657
    public static void pickDirectory() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
        // try {
        //     mSingleton.startActivityForResult(intent, requestCode);
        // } catch (ActivityNotFoundException e) {
        //     Log.e(TAG, "Unable to open file dialog.", e);
        //     return false;
        // }

        mSingleton.startActivityForResult(intent, 0); // request code guess, SDL just increments it, so it might not
                                                      // work in general case
        ((MyActivity) mSingleton).pickDir = true;
    }


    public static String createDirectory(String parent, String dir) {
        Uri uri = Uri.parse(parent);
        DocumentFile parentDir = DocumentFile.fromTreeUri(mSingleton, uri);
        DocumentFile file = parentDir.findFile(dir);
        if (file == null) {
            file = parentDir.createDirectory(dir);
        }
        return file.getUri().toString();
    }

    public static String findFile(String parent, String filename) {
        Uri uri = Uri.parse(parent);
        DocumentFile parentDir = DocumentFile.fromTreeUri(mSingleton, uri);
        DocumentFile file = parentDir.findFile(filename);
        if (file == null) {
            return null;
        }
        return file.getUri().toString();
    }
    
    // Returns array of (fileUri, fileName)
    public static String[][] readDir(String dir) {
        Uri uri = Uri.parse(dir);
        DocumentFile root = DocumentFile.fromTreeUri(mSingleton, uri);

        String[][] fileList = null;
        if (root != null && root.isDirectory()) {
            ArrayList<String[]> fileListArr = new ArrayList();
            for (DocumentFile file : root.listFiles()) {
                if (!file.isDirectory()) {
                    fileListArr.add(new String[]{file.getUri().toString(), file.getName()});
                }
            }
            fileList = new String[fileListArr.size()][2];
            fileList = fileListArr.toArray(fileList);
        }

        return fileList;
    }
}
